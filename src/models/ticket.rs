
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::fmt;
use crate::models::appstate::AppState;

// Enum para o Status do Ticket
#[derive(Debug, Deserialize, Serialize, sqlx::Type, Clone, Copy)]
#[sqlx(type_name = "StatusTicket", rename_all = "PascalCase")]
pub enum StatusTicket { Aberto, Fechado }

impl fmt::Display for StatusTicket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusTicket::Aberto  => write!(f, "Aberto"),
            StatusTicket::Fechado => write!(f, "Fechado"),
        }
    }
}

// Enum para a Prioridade do Ticket
#[derive(Debug, Deserialize, Serialize, sqlx::Type, Clone, Copy)]
#[sqlx(type_name = "Priority", rename_all = "lowercase")] // O banco espera 'baixa', 'média', 'alta'
pub enum Priority { Baixa, Média, Alta }

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Para exibição no frontend, usamos PascalCase
        match self {
            Priority::Baixa => write!(f, "Baixa"),
            Priority::Média => write!(f, "Média"),
            Priority::Alta  => write!(f, "Alta"),
        }
    }
}

// Enum para a Categoria do Ticket
#[derive(Debug, Deserialize, Serialize, sqlx::Type, Clone, Copy)]
#[sqlx(type_name = "Category", rename_all = "PascalCase")]
pub enum Category { Software, Hardware, Redes, Acesso }

impl fmt::Display for Category { 
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Software => write!(f, "Software"),
            Category::Hardware => write!(f, "Hardware"),
            Category::Redes    => write!(f, "Redes"),
            Category::Acesso   => write!(f, "Acesso"),
        }
    }
}

// Estrutura para representar um Ticket
#[derive(Debug, Deserialize, Serialize)]
pub struct Ticket {
    pub ticket_id: Option<i32>,
    pub ticket_status: StatusTicket,
    pub ticket_priority: Priority,
    pub ticket_description: String,
    pub ticket_title: String,
    pub ticket_client_id: i32,
    pub ticket_category: Category,
    pub ticket_client_name: String,
}

// Estrutura para criar um novo Ticket
#[derive(Debug, Deserialize, Serialize)]
pub struct NewTicket {
    pub ticket_description: String,
    pub ticket_client_id: i32,
    pub ticket_title: String,
    pub ticket_category: Category,
}

impl Ticket {
    // Construtor para uma nova instância de Ticket (principalmente para uso interno ou testes)
    pub fn new_ticket(
        title: String,
        description: String,
        client_id: i32,
        category: Category,
        client_name: String,
    ) -> Self {
        Self {
            ticket_id: None,
            ticket_status: StatusTicket::Aberto,
            ticket_priority: Priority::Média, // Prioridade padrão ao criar localmente
            ticket_description: description,
            ticket_client_id: client_id,
            ticket_title: title,
            ticket_category: category,
            ticket_client_name: client_name,
        }
    }

    // Salva um novo ticket no banco de dados
    pub async fn save_new_ticket_in_db(
        new_ticket_payload: &NewTicket,
        state: Arc<AppState>,
    ) -> Result<Ticket, sqlx::Error> {
        let query_sql = "INSERT INTO Tickets (
                Ticket_Title, Ticket_Description, ID_User_Requesting,
                Ticket_category, Ticket_Status, Ticket_Priority
            ) VALUES (?, ?, ?, ?, ?, ?)";

        let default_status = StatusTicket::Aberto;
        let default_priority = Priority::Média; // Prioridade padrão ao salvar no banco

        let result = sqlx::query(query_sql)
            .bind(&new_ticket_payload.ticket_title)
            .bind(&new_ticket_payload.ticket_description)
            .bind(new_ticket_payload.ticket_client_id)
            .bind(new_ticket_payload.ticket_category.to_string()) // "Software", "Hardware", etc.
            .bind(default_status.to_string())                     // "Aberto"
            .bind(default_priority.to_string().to_lowercase())    // Salva como "baixa", "média", "alta"
            .execute(&state.pool)
            .await?;

        let last_inserted_id = result.last_insert_id();

        let client_name_from_db: String = sqlx::query_scalar(
            "SELECT User_Name FROM Users WHERE ID_User = ?"
        )
        .bind(new_ticket_payload.ticket_client_id)
        .fetch_one(&state.pool)
        .await
        .unwrap_or_else(|_| {
            // Em produção, logar o erro 'e' aqui.
            "Cliente Desconhecido".to_string() // Fallback
        });

        Ok(Ticket {
            ticket_id: Some(last_inserted_id as i32),
            ticket_title: new_ticket_payload.ticket_title.clone(),
            ticket_description: new_ticket_payload.ticket_description.clone(),
            ticket_client_id: new_ticket_payload.ticket_client_id,
            ticket_category: new_ticket_payload.ticket_category,
            ticket_status: default_status,
            ticket_priority: default_priority, // Retorna o enum; a exibição usará Display
            ticket_client_name: client_name_from_db,
        })
    }

    // Busca os tickets abertos de um utilizador específico
    pub async fn get_open_tickets(
        state: Arc<AppState>,
        current_user_id: i32,
    ) -> Result<Vec<Ticket>, sqlx::Error> {
        let query_sql = "SELECT
                t.ID_Ticket, t.Ticket_Title, t.Ticket_Status, t.Ticket_Priority,
                t.Ticket_category, t.Ticket_Description, t.ID_User_Requesting,
                u.User_Name AS client_name_from_db
            FROM Tickets t
            JOIN Users u ON t.ID_User_Requesting = u.ID_User
            WHERE t.Ticket_Status = 'Aberto' AND t.ID_User_Requesting = ?";

        let rows = sqlx::query(query_sql)
            .bind(current_user_id)
            .fetch_all(&state.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                let category_str: String = row.try_get("Ticket_category")?;
                let resolved_category = match category_str.as_str() {
                    "Software" => Category::Software, "Hardware" => Category::Hardware,
                    "Redes" => Category::Redes, "Acesso" => Category::Acesso,
                    _ => Category::Software // Fallback
                };

                let status_str: String = row.try_get("Ticket_Status")?;
                let resolved_status = match status_str.as_str() {
                    "Aberto" => StatusTicket::Aberto, "Fechado" => StatusTicket::Fechado,
                    _ => StatusTicket::Aberto // Fallback
                };

                let priority_str: String = row.try_get("Ticket_Priority")?; // Vem como 'baixa', 'média', 'alta'
                let resolved_priority = match priority_str.to_lowercase().as_str() {
                    "baixa" => Priority::Baixa, 
                    "média" | "media" => Priority::Média, // Aceita "media" sem acento também
                    "alta" => Priority::Alta,
                    _ => Priority::Média // Fallback
                };

                Ok(Ticket {
                    ticket_id: row.try_get("ID_Ticket")?, // ID é importante, não usar _for_log aqui
                    ticket_title: row.try_get("Ticket_Title")?,
                    ticket_status: resolved_status,
                    ticket_priority: resolved_priority,
                    ticket_category: resolved_category,
                    ticket_description: row.try_get("Ticket_Description")?,
                    ticket_client_id: row.try_get("ID_User_Requesting")?,
                    ticket_client_name: row.try_get("client_name_from_db")?,
                })
            })
            .collect() // Coleta em Result<Vec<Ticket>, sqlx::Error>
    }

    // Busca todos os tickets (para admin)
    pub async fn get_all_tickets(state: Arc<AppState>) -> Result<Vec<Ticket>, sqlx::Error> {
        let query_sql = "SELECT
                t.ID_Ticket, t.Ticket_Title, t.Ticket_Status, t.Ticket_Priority,
                t.Ticket_category, t.Ticket_Description, t.ID_User_Requesting,
                u.User_Name AS client_name_from_db
            FROM Tickets t
            JOIN Users u ON t.ID_User_Requesting = u.ID_User";
        
        let rows = sqlx::query(query_sql)
            .fetch_all(&state.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                let category_str: String = row.try_get("Ticket_category")?;
                let resolved_category = match category_str.as_str() {
                    "Software" => Category::Software, "Hardware" => Category::Hardware,
                    "Redes" => Category::Redes, "Acesso" => Category::Acesso,
                    _ => Category::Software // Fallback
                };

                let status_str: String = row.try_get("Ticket_Status")?;
                let resolved_status = match status_str.as_str() {
                    "Aberto" => StatusTicket::Aberto, "Fechado" => StatusTicket::Fechado,
                    _ => StatusTicket::Aberto // Fallback
                };

                let priority_str: String = row.try_get("Ticket_Priority")?;
                let resolved_priority = match priority_str.to_lowercase().as_str() {
                    "baixa" => Priority::Baixa, 
                    "média" | "media" => Priority::Média,
                    "alta" => Priority::Alta,
                    _ => Priority::Média // Fallback
                };
                
                Ok(Ticket {
                    ticket_id: row.try_get("ID_Ticket")?,
                    ticket_title: row.try_get("Ticket_Title")?,
                    ticket_status: resolved_status,
                    ticket_priority: resolved_priority,
                    ticket_category: resolved_category,
                    ticket_description: row.try_get("Ticket_Description")?,
                    ticket_client_id: row.try_get("ID_User_Requesting")?,
                    ticket_client_name: row.try_get("client_name_from_db")?,
                })
            })
            .collect()
    }

    // Atualiza a prioridade de um ticket específico
    pub async fn update_ticket_priority(
        state: Arc<AppState>,
        ticket_id: i32,
        new_priority: Priority,
    ) -> Result<sqlx::mysql::MySqlQueryResult, sqlx::Error> {
        let query_sql = "UPDATE Tickets SET Ticket_Priority = ? WHERE ID_Ticket = ?";
        
        // Converte o enum para a string que o DB espera (lowercase, conforme rename_all)
        let priority_str_for_db = match new_priority {
            Priority::Baixa => "baixa",
            Priority::Média => "média", // Se o DB armazena com acento
            Priority::Alta  => "alta",
        };

        sqlx::query(query_sql)
            .bind(priority_str_for_db)
            .bind(ticket_id)
            .execute(&state.pool)
            .await
    }
}
