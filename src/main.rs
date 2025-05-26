use std::sync::Arc;
use axum::{
    response::{Html, IntoResponse}, 
    routing::get,
    extract::State,
    Router
};
use tera::Tera;
use sqlx::mysql::MySqlPool;
use dotenv::dotenv;
use models::appstate::AppState; // Assumindo que AppState é acessível via 'models'
use tower_http::services::ServeDir;

// Declaração dos módulos principais da aplicação
mod auth;      
mod controllers; 
mod db;          
mod models;      
mod routes;      
mod services;    

#[tokio::main]
async fn main() {
    dotenv().ok(); // Carrega variáveis de ambiente do .env

    // Configuração do estado da aplicação
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("Falha ao carregar JWT_SECRET do ambiente");
    let tera = Tera::new("./src/templates/**/*.html")
        .expect("Falha ao carregar templates Tera");
    let pool: MySqlPool = db::connection::establish_connection()
        .await
        .expect("Falha ao estabelecer conexão com o banco de dados");

    let state = Arc::new(AppState {
        tera,
        pool,
        jwt_secret,
    });

    // Definição das rotas da aplicação
    let app = Router::new()
        .nest_service("/static", ServeDir::new("src/static")) // Serve ficheiros estáticos
        .route("/", get(render_index_page)) // Rota para a página inicial
        // Rotas específicas da aplicação aninhadas
        .nest("/register", crate::routes::register::RegisterRoute::create_register_route(state.clone()))
        .nest("/login", crate::routes::auth::AuthRoute::get_authenticated(state.clone()))
        .nest("/new_ticket", crate::routes::ticket::TicketRoute::create_new_ticket_route(state.clone()))
        .nest(
            "/tickets", // Agrupa rotas relacionadas a tickets existentes
            crate::routes::ticket::TicketRoute::list_tickets_route(state.clone())
                .merge(crate::routes::ticket::TicketRoute::close_ticket_route(state.clone()))
        )
        .nest("/admin", crate::routes::admin::AdminRoute::create_admin_routes(state.clone()))
        .with_state(state); // Aplica o estado compartilhado a todas as rotas
    
    // Inicia o servidor
    let listener = tokio::net::TcpListener::bind("localhost:8080")
        .await
        .expect("Falha ao iniciar o listener TCP");
    
    // Log de início do servidor (opcional, mas útil)
    // Se quiser remover todos os println!, pode remover este também.
    // Por enquanto, vou mantê-lo como um log útil de que o servidor iniciou.
    println!("Servidor a rodar em {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .expect("Falha ao iniciar o servidor Axum");
}

// Handler para renderizar a página de índice.
async fn render_index_page(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // println!("GET /index"); // Removido
    match state.tera.render("index.html", &tera::Context::new()) {
        Ok(rendered_html) => Html(rendered_html),
        Err(_) => {
            // Em produção, logar o erro 'e' detalhado.
            Html("<h1>Erro ao carregar a página inicial</h1>".to_string())
        }
    }
}
