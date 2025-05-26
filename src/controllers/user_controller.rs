use std::sync::Arc;
use std::str::FromStr;
use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect},
    http::StatusCode,
};
use serde::Deserialize;
use tera::Context;

// Importações dos modelos e serviços
use crate::models::{
    appstate::AppState,
    user::{User, UserRole},
};
use crate::services::auth_service::password_hash;

// Estrutura para os dados do formulário de registro
#[derive(Deserialize, Debug)]
pub struct RegisterPayload {
    user_name: String,
    user_email: String,
    user_password: String,
    user_role: String, // Recebe "Cliente" ou "Administrador"
}

// Handler para criar um novo utilizador (POST /register)
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<RegisterPayload>,
) -> impl IntoResponse {
    let role_from_form = match UserRole::from_str(&payload.user_role) {
        Ok(r) => r,
        Err(_) => {
            // Em produção, este erro deve ser logado.
            // Para o utilizador, poderia retornar um erro mais específico ou
            // renderizar o formulário novamente com uma mensagem.
            // Mantendo o fallback para Cliente, mas idealmente validaria no frontend/retornaria erro.
            UserRole::Cliente 
        }
    };
    
    // Lógica de segurança para registo de Admin pode ser adicionada aqui em produção.

    let hashed_password = password_hash(&payload.user_password);

    let new_user = User::build_user(
        payload.user_name.clone(),
        payload.user_email.clone(),
        hashed_password,
        role_from_form,
    );

    match User::save_user_in_db(&new_user, Arc::clone(&state)).await {
        Ok(_) => {
            Redirect::to("/login?success=1").into_response()
        }
        Err(e) => {
            // Em produção, o erro 'e' deve ser logado detalhadamente.
            let mut error_context = Context::new();
            error_context.insert("static_path", "/static");
            error_context.insert("user_name_val", &payload.user_name);
            error_context.insert("user_email_val", &payload.user_email);
            // A senha não é repopulada por segurança.

            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() { // Verifica se é um erro de violação de unicidade (ex: email duplicado)
                    error_context.insert("error_message", "Este e-mail já está registado. Tente outro.");
                    let rendered = state.tera.render("register.html", &error_context)
                        .unwrap_or_else(|_| "Erro: E-mail já registado.".to_string()); // Fallback se o template de erro falhar
                    return (StatusCode::CONFLICT, Html(rendered)).into_response();
                }
            }
            
            // Erro genérico se não for uma violação de unicidade
            error_context.insert("error_message", "Ocorreu um erro ao tentar registar. Tente novamente mais tarde.");
            let rendered = state.tera.render("register.html", &error_context)
                .unwrap_or_else(|_| "Erro interno no servidor.".to_string()); // Fallback
            (StatusCode::INTERNAL_SERVER_ERROR, Html(rendered)).into_response()
        }
    }
}

// Handler para renderizar a página de registo (GET /register)
pub async fn render_register_page(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("static_path", "/static");
    // Valores iniciais para os campos do formulário (para repopulação ou estado inicial)
    context.insert("user_name_val", "");
    context.insert("user_email_val", "");
    context.insert("error_message", ""); // Para evitar erro no template se a variável não existir

    match state.tera.render("register.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(_) => {
            // Em produção, o erro 'e' deve ser logado.
            (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Erro ao carregar página de registo</h1>".to_string())).into_response()
        }
    }
}
