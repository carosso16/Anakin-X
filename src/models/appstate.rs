use tera::Tera;
use sqlx::mysql::MySqlPool;

pub struct AppState {
    pub tera: Tera,
    pub pool: MySqlPool,
    pub jwt_secret: String,
}
