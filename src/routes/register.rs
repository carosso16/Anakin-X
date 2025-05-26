use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
};
use crate::{
    // Importa os handlers do controller de utilizador (user_controller)
    controllers::user_controller::{create_user, render_register_page}, 
    models::appstate::AppState,
};

pub struct RegisterRoute;

impl RegisterRoute {
    // Cria e retorna as rotas de registo.
    pub fn create_register_route(state: Arc<AppState>) -> Router<Arc<AppState>> {
        Router::new()
            // Define a rota raiz ("/") para:
            // - GET: renderizar a página de registo.
            // - POST: processar a criação de um novo utilizador.
            .route("/", get(render_register_page).post(create_user))
            .with_state(state)
    }
}
