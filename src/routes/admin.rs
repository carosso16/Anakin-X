
use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
};
use crate::{
    controllers::admin_controller::{
        admin_dashboard_page_handler, 
        get_admin_dashboard_data_api, 
        set_ticket_priority_handler
    },
    models::appstate::AppState,
};

pub struct AdminRoute;

impl AdminRoute {
    // Cria as rotas específicas para a área de administração.
    pub fn create_admin_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
        Router::new()
            // Rota para exibir o painel principal do administrador.
            .route("/dashboard", get(admin_dashboard_page_handler))
            // Rota de API para buscar os dados a serem exibidos no dashboard do admin.
            .route("/dashboard-data", get(get_admin_dashboard_data_api))
            // Rota para o administrador definir a prioridade de um ticket específico.
            .route("/tickets/:id/set-priority", post(set_ticket_priority_handler))
            .with_state(state)
    }
}
