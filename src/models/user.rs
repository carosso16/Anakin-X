// src/models/user.rs

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, Type}; // Removido 'query' não utilizado do import
use std::str::FromStr;
use std::fmt;
use crate::models::appstate::AppState;

// Estrutura para erro de parsing de UserRole
#[derive(Debug)]
struct UserRoleParseError(String);

impl fmt::Display for UserRoleParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0) // Mantida a formatação original do erro
    }
}

impl std::error::Error for UserRoleParseError {}

// Enum para o Papel do Utilizador
#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "User_Role")] // Mapeia para o tipo ENUM 'User_Role' no banco de dados
pub enum UserRole {
    Cliente,
    Administrador,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::Cliente => write!(f, "Cliente"),
            UserRole::Administrador => write!(f, "Administrador"),
        }
    }
}

impl FromStr for UserRole {
    type Err = String; // O erro ao converter de string é uma String
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cliente" => Ok(UserRole::Cliente),
            "administrador" => Ok(UserRole::Administrador),
            _ => Err(format!("Valor inválido para UserRole: {}", s)),
        }
    }
}

// Estrutura para representar um Utilizador
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    #[sqlx(rename = "ID_User")] 
    pub user_id: Option<i32>,
    #[sqlx(rename = "User_Name")]
    pub user_name: String,
    #[sqlx(rename = "User_Email")]
    pub user_email: String,
    #[sqlx(rename = "User_Password")]
    pub user_password: String, // Deve armazenar a senha com hash
    #[sqlx(rename = "User_Role")]
    pub user_role: UserRole,
}

impl User {
    // Construtor para uma nova instância de User
    pub fn build_user(name: String, email: String, password: String, role: UserRole) -> Self {
        Self {
            user_id: None,
            user_name: name,
            user_email: email,
            user_password: password, // Importante: esta senha já deve estar com hash
            user_role: role,
        }
    }

    // Salva um novo utilizador no banco de dados
    pub async fn save_user_in_db(new_user: &User, state: Arc<AppState>) -> Result<sqlx::mysql::MySqlQueryResult, sqlx::Error> {
        sqlx::query(
            "INSERT INTO Users (User_Name, User_Email, User_Password, User_Role) VALUES (?, ?, ?, ?)"
        )
        .bind(&new_user.user_name)
        .bind(&new_user.user_email)
        .bind(&new_user.user_password) // Assume que new_user.user_password já está com hash
        .bind(new_user.user_role.to_string())
        .execute(&state.pool)
        .await
    }

    // Busca um utilizador pelo email
    pub async fn get_user_by_email(email: String, state: Arc<AppState>) -> Result<User, sqlx::Error> {
        let row = sqlx::query(
            "SELECT ID_User, User_Name, User_Email, User_Password, User_Role FROM Users WHERE User_Email = ?"
        )
        .bind(email)
        .fetch_one(&state.pool)
        .await?;
    
        let user_role_str: String = row.try_get("User_Role")?;
        // Converte a string do banco para o enum UserRole, tratando possíveis erros de decodificação
        let user_role_enum = UserRole::from_str(&user_role_str)
            .map_err(|e_str| sqlx::Error::Decode(Box::new(UserRoleParseError(e_str))))?;

        Ok(User {
            user_id: row.try_get("ID_User")?,
            user_name: row.try_get("User_Name")?,
            user_email: row.try_get("User_Email")?,
            user_password: row.try_get("User_Password")?,
            user_role: user_role_enum,
        })
    }
}
