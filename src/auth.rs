
use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Serialize;
use std::sync::Arc;

// Importa as structs Claims e AppState dos seus respectivos módulos.
use crate::services::auth_service::Claims; 
use crate::models::appstate::AppState;

// Estrutura para erros de autenticação retornados como JSON.
#[derive(Serialize)]
pub struct AuthError {
    message: String,
}

// Extractor para obter as Claims de um utilizador autenticado a partir de um JWT.
pub struct AuthUser(pub Claims);

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = Response; // O tipo de rejeição é uma Response completa.

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Tenta obter o valor do cabeçalho Authorization.
        let auth_header_value = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        // Extrai a string do token do cabeçalho.
        let token_str = match auth_header_value {
            Some(value_str) => {
                // Verifica se o cabeçalho começa com "Bearer " e remove este prefixo.
                if value_str.starts_with("Bearer ") {
                    value_str.trim_start_matches("Bearer ").to_owned()
                } else {
                    // Se não for do tipo Bearer, retorna erro 401.
                    let body = Json(AuthError {
                        message: "Token de autorização mal formatado (requer prefixo 'Bearer ').".to_string(),
                    });
                    return Err((StatusCode::UNAUTHORIZED, body).into_response());
                }
            }
            None => {
                // Se o cabeçalho Authorization estiver ausente, retorna erro 401.
                let body = Json(AuthError {
                    message: "Token de autorização ausente.".to_string(),
                });
                return Err((StatusCode::UNAUTHORIZED, body).into_response());
            }
        };

        // Decodifica e valida o token JWT.
        let token_data = decode::<Claims>(
            &token_str,
            &DecodingKey::from_secret(state.jwt_secret.as_ref()),
            &Validation::default(), // Utiliza validações padrão (algoritmo, expiração).
        )
        .map_err(|jwt_error| {
            // Em caso de erro na decodificação/validação, determina a mensagem e retorna 401.
            // Em produção, o 'jwt_error' detalhado deve ser logado.
            let error_message = match jwt_error.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Token expirado.",
                jsonwebtoken::errors::ErrorKind::InvalidToken => "Token inválido.",
                jsonwebtoken::errors::ErrorKind::InvalidSignature => "Assinatura do token inválida.",
                _ => "Erro de autenticação. Token não pôde ser validado.",
            };
            let body = Json(AuthError {
                message: error_message.to_string(),
            });
            (StatusCode::UNAUTHORIZED, body).into_response()
        })?;

        // Se tudo estiver OK, retorna as claims encapsuladas em AuthUser.
        Ok(AuthUser(token_data.claims))
    }
}
