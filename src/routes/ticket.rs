use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router
};
use crate::{
    controllers::ticket_controller::{
        create_ticket, 
        list_tickets,
        new_ticket,
        close_ticket,
        get_my_open_tickets_api
    },
    models::appstate::AppState
};

pub struct TicketRoute;

impl TicketRoute {
    // Define as rotas relacionadas à criação e visualização de tickets de utilizador.
    pub fn create_new_ticket_route(state: Arc<AppState>) -> Router<Arc<AppState>> {
        Router::new()
            // GET /: Serve a página HTML base para abrir um novo ticket (onde os tickets do utilizador são carregados via API).
            // POST /: Processa a submissão do formulário para criar um novo ticket.
            .route("/", get(new_ticket).post(create_ticket)) 
            // GET /api/my-open-tickets: Endpoint de API para o frontend buscar os tickets abertos do utilizador logado.
            .route("/api/my-open-tickets", get(get_my_open_tickets_api))
            .with_state(state)
    }

    // Define a rota para listar os tickets (ex: tickets abertos do utilizador).
    // O handler 'list_tickets' é protegido e espera informações do utilizador autenticado.
    pub fn list_tickets_route(state: Arc<AppState>) -> Router<Arc<AppState>> {
        Router::new()
            .route("/", get(list_tickets))
            .with_state(state)
    }

    // Define a rota para fechar um ticket específico.
    pub fn close_ticket_route(state: Arc<AppState>) -> Router<Arc<AppState>> {
        Router::new()
            .route("/:id/close", post(close_ticket))
            .with_state(state)
    }
}
