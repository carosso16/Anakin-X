use sqlx::mysql::MySqlPool;
use dotenv::dotenv;
use std::env;

// cria uma função para estabelecer conexao com o banco de dados mysql atraves do sqlx
pub async fn establish_connection() -> Result<MySqlPool, sqlx::Error> {
    // carrega as variaveis de ambiente do arquivo .env
    dotenv().ok();
    let user = env::var("USER").expect("USER user precisa estar configurado!");
    let password = env::var("PASSWORD").expect("PASSWORD user precisa estar configurada!");
    let server = env::var("SERVER").expect("SERVER user precisa estar configurado!");
    let port = env::var("PORT").expect("PORT user precisa estar configurado!");
    let database = env::var("DATABASE").expect("DATABASE user precisa estar configurado!");

    // cria a string url de acesso ao banco ja com as variaveis
    let database_url = format!("mysql://{}:{}@{}:{}/{}", user, password, server, port, database);

    // cria uma pool de conexao, se der certo atribui o estado 
    let pool = MySqlPool::connect(&database_url).await?;
    Ok(pool)
}

