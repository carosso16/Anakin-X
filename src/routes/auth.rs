use std::sync::Arc;
use std::collections::HashMap; // Necessário para Query<HashMap<String, String>>
use axum::{
    extract::{State, Query},
    response::{Html, IntoResponse}, // Html é usado para a resposta de erro
    routing::{get, post},
    Router,
    http::StatusCode, // Importe StatusCode
};
use tera; // Para tera::Context

// Importe o AppState e o handler de login do controller
use crate::models::appstate::AppState;
use crate::controllers; // Usado para controllers::auth_controller::login

pub struct AuthRoute;

impl AuthRoute {
    // Cria as rotas para autenticação (login).
    pub fn get_authenticated(state: Arc<AppState>) -> Router<Arc<AppState>> {
        Router::new()
            // Rota para exibir a página de login.
            .route("/", get(render_login_page))
            // Rota para processar a submissão do formulário de login.
            .route("/", post(controllers::auth_controller::login))
            .with_state(state)
    }
}

// Handler para renderizar a página de login (GET /login).
pub async fn render_login_page(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>, // Parâmetros da query, como '?success=1'
) -> impl IntoResponse {
    let template_name: &str = "login.html";
    let mut context = tera::Context::new();

    context.insert("static_path", "/static");

    // Verifica se há um parâmetro 'success' na URL (ex: após registo bem-sucedido).
    let success_value = params.get("success").map_or("", |s| s.as_str());
    context.insert("success", success_value);

    match state.tera.render(template_name, &context) {
        Ok(rendered_html) => Html(rendered_html).into_response(),
        Err(_) => {
            // Em produção, logar o erro 'e' detalhado.
            // Retornar uma página de erro HTML simples.
            (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Erro ao carregar a página de login.</h1>".to_string())).into_response()
        }
    }
}
