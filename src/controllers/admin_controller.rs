use std::sync::Arc;
use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use serde_json; // Para serde_json::json! e serde_json::to_string
use tera::Context;

use crate::{
    models::{
        appstate::AppState,
        ticket::{Ticket, Priority},
        user::UserRole,
    },
    auth::AuthUser,
};

// Handler para servir a página HTML base do painel de admin
pub async fn admin_dashboard_page_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("static_path", "/static");

    let priorities_options_for_js = vec![
        Priority::Baixa.to_string(),
        Priority::Média.to_string(),
        Priority::Alta.to_string(),
    ];
    // Prepara as opções de prioridade como uma string JSON para o template
    context.insert(
        "priorities_options_json", 
        &serde_json::to_string(&priorities_options_for_js).unwrap_or_else(|_| "[]".to_string())
    );

    match state.tera.render("admin_dashboard.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(_) => {
            // Em uma aplicação real, logar o erro 'e' aqui seria importante
            (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Erro ao renderizar página de admin</h1>".to_string())).into_response()
        }
    }
}

// Handler de API para buscar os dados do dashboard do admin
pub async fn get_admin_dashboard_data_api(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser, // Protegido
) -> impl IntoResponse {
    if claims.role != UserRole::Administrador.to_string() {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"erro": "Acesso Negado. Somente administradores."}))).into_response();
    }

    match Ticket::get_all_tickets(state.clone()).await {
        Ok(tickets) => {
            (StatusCode::OK, Json(serde_json::json!({ "tickets": tickets }))).into_response()
        }
        Err(_) => {
            // Logar o erro 'e'
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"erro": "Erro ao carregar dados dos tickets"}))).into_response()
        }
    }
}

// Struct para o payload de definir prioridade
#[derive(Deserialize, Debug)]
pub struct SetPriorityPayload {
    priority: String, // Espera "Baixa", "Média", ou "Alta"
}

// Handler para definir a prioridade de um ticket
pub async fn set_ticket_priority_handler(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser, // Protegido
    Path(ticket_id): Path<i32>,
    Json(payload): Json<SetPriorityPayload>,
) -> impl IntoResponse {
    if claims.role != UserRole::Administrador.to_string() {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"erro": "Acesso Negado."}))).into_response();
    }

    // Converte a string de prioridade do payload para o enum Priority
    let new_priority_enum = match payload.priority.as_str() {
        "Baixa" => Priority::Baixa,
        "Média" => Priority::Média,
        "Alta" => Priority::Alta,
        _ => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"erro": "Valor de prioridade inválido"}))).into_response();
        }
    };

    match Ticket::update_ticket_priority(state, ticket_id, new_priority_enum).await {
        Ok(result) => {
            if result.rows_affected() > 0 {
                (StatusCode::OK, Json(serde_json::json!({"mensagem": "Prioridade atualizada com sucesso"}))).into_response()
            } else {
                (StatusCode::NOT_FOUND, Json(serde_json::json!({"erro": "Ticket não encontrado"}))).into_response()
            }
        }
        Err(_) => {
            // Logar o erro 'e'
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"erro": "Erro interno ao atualizar prioridade"}))).into_response()
        }
    }
}
