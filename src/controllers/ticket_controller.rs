use std::sync::Arc;
use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use serde_json::json; // Para a macro json!
use tera::Context;

// Importações dos modelos e autenticação
use crate::models::{
    appstate::AppState,
    ticket::{NewTicket, Ticket},
};
use crate::auth::AuthUser; // Extractor para utilizador autenticado

// Handler para criar um novo ticket (POST /new_ticket)
pub async fn create_ticket(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser, // Requer autenticação
    Json(mut payload): Json<NewTicket>, // Payload do novo ticket
) -> impl IntoResponse {
    let user_id_str = claims.sub;
    let current_user_id = match user_id_str.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            // Em produção, logar este erro.
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"erro": "ID de utilizador inválido no token."}))
            ).into_response();
        }
    };

    // Define o ticket_client_id com o ID do utilizador autenticado
    payload.ticket_client_id = current_user_id;

    match Ticket::save_new_ticket_in_db(&payload, state).await {
        Ok(ticket_criado_no_db) => {
            (StatusCode::CREATED, Json(ticket_criado_no_db)).into_response()
        }
        Err(_) => {
            // Em produção, logar o erro 'e' detalhado.
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": "Erro ao salvar ticket"}))).into_response()
        }
    }
}

// Handler para listar os tickets abertos do utilizador autenticado (GET /tickets)
// Esta rota pode precisar de um nome mais específico se houver uma rota de admin para todos os tickets.
pub async fn list_tickets(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser, // Requer autenticação
) -> Result<Json<Vec<Ticket>>, (StatusCode, Json<serde_json::Value>)> { // Tipo de retorno mais explícito
    let user_id_str = claims.sub;
    let current_user_id = match user_id_str.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            // Em produção, logar este erro.
            return Err((
                StatusCode::BAD_REQUEST, // Alterado para BAD_REQUEST pois o token está malformado para este contexto
                Json(json!({"erro": "ID de utilizador inválido no token."}))
            ));
        }
    };

    match Ticket::get_open_tickets(state, current_user_id).await {
        Ok(tickets) => Ok(Json(tickets)),
        Err(_) => {
            // Em produção, logar o erro 'e' detalhado.
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"erro": "Erro ao buscar os seus tickets."}))
            ))
        }
    }
}

// Handler para a API que fornece os tickets abertos do utilizador para carregamento dinâmico
pub async fn get_my_open_tickets_api(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser, // Requer autenticação
) -> Result<Json<Vec<Ticket>>, (StatusCode, Json<serde_json::Value>)> { // Tipo de retorno mais explícito
    let user_id_str = claims.sub;
    let current_user_id = match user_id_str.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            // Em produção, logar este erro.
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"erro": "ID de utilizador inválido no token."}))
            ));
        }
    };

    match Ticket::get_open_tickets(state.clone(), current_user_id).await {
        Ok(tickets) => Ok(Json(tickets)),
        Err(_) => {
            // Em produção, logar o erro 'e' detalhado.
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"erro": "Erro ao buscar os seus tickets."}))
            ))
        }
    }
}

// Handler para servir a página HTML base de "novo ticket" (GET /new_ticket)
// Esta página carregará os tickets dinamicamente via JavaScript.
pub async fn new_ticket(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("static_path", "/static"); 

    match state.tera.render("new_ticket.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(_) => {
            // Em produção, logar o erro 'e' detalhado.
            (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Erro ao renderizar página</h1>".to_string())).into_response()
        }
    }
}

// Handler para fechar um ticket (POST /tickets/:id/close)
pub async fn close_ticket(
    Path(id): Path<i32>, // ID do ticket a ser fechado
    State(state): State<Arc<AppState>>,
    AuthUser(_claims): AuthUser, // Requer autenticação, mas não usamos as claims diretamente aqui (a menos que queira validar se o utilizador pode fechar este ticket)
) -> impl IntoResponse {
    // Para MySQL ou SQLite
    let query_sql = "UPDATE Tickets SET Ticket_Status = 'Fechado' WHERE ID_Ticket = ?";

    match sqlx::query(query_sql)
        .bind(id)
        .execute(&state.pool)
        .await 
    {
        Ok(query_result) => {
            if query_result.rows_affected() > 0 {
                (StatusCode::OK, Json(json!({"mensagem": "Chamado fechado com sucesso"}))).into_response()
            } else {
                (StatusCode::NOT_FOUND, Json(json!({"erro": "Chamado não encontrado"}))).into_response()
            }
        }
        Err(_) => {
            // Em produção, logar o erro 'e' detalhado.
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"erro": "Erro ao fechar chamado"}))).into_response()
        }
    }
}
