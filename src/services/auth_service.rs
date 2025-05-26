use chrono::{Duration, Utc};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use crate::models::user::UserRole; // Para o tipo UserRole nas claims
use bcrypt::{hash, verify, DEFAULT_COST};

// Estrutura das claims contidas no JWT.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // Subject (ID do utilizador)
    pub role: String,   // Papel do utilizador (ex: "Cliente", "Administrador")
    pub exp: usize,     // Timestamp de expiração do token
}

// Gera um hash bcrypt para uma senha.
pub fn password_hash(password: &str) -> String {
    hash(password, DEFAULT_COST).expect("Falha ao gerar hash da senha")
}

// Verifica se uma senha corresponde a um hash bcrypt.
pub fn password_verify(password: &str, hashed_password: &str) -> bool {
    verify(password, hashed_password).unwrap_or(false)
}

// Gera um token JWT para um utilizador.
pub fn jwt_gen(secret_key: &str, user_id: i32, user_role: &UserRole) -> String {
    let claims = Claims {
        sub: user_id.to_string(),
        role: user_role.to_string(), // Utiliza a implementação Display de UserRole
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_bytes())
    )
    .expect("Falha ao gerar token JWT")
}

// Decodifica e valida um token JWT, retornando as claims se for válido.
pub fn jwt_decode_and_validate(secret_key: &str, token: &str) -> Result<Claims, JwtError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_key.as_bytes()),
        &Validation::default(), // Utiliza validações padrão (algoritmo, expiração)
    ).map(|token_data| token_data.claims)
}
