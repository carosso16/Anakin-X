use std::sync::Arc;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::models::appstate::AppState;
use crate::models::user::User;
use crate::services::auth_service::{jwt_gen, password_verify};

#[derive(Debug, Deserialize)]
pub struct LoginData {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct ErrorMessage {
    error: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(data): Json<LoginData>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<ErrorMessage>)> {
    let user_result = User::get_user_by_email(data.email.clone(), state.clone()).await;

    match user_result {
        Ok(user) => {
            if password_verify(&data.password, &user.user_password) {
                let user_id = match user.user_id {
                    Some(id) => id,
                    None => {
                        // Em produção, logar este erro criticamente.
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorMessage {
                                error: "Erro interno ao processar informações do utilizador.".to_string(),
                            }),
                        ));
                    }
                };

                let token = jwt_gen(&state.jwt_secret, user_id, &user.user_role);
                Ok(Json(TokenResponse {
                    token,
                    role: user.user_role.to_string(),
                }))
            } else {
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorMessage {
                        error: "Credenciais inválidas".to_string(),
                    }),
                ))
            }
        }
        Err(err) => {
            // Em produção, logar o 'err' detalhado.
            if matches!(err, sqlx::Error::RowNotFound) {
                 Err((
                    StatusCode::UNAUTHORIZED, // Ou NOT_FOUND, mas UNAUTHORIZED é comum para falha de login
                    Json(ErrorMessage {
                        error: "Utilizador não encontrado ou credenciais inválidas.".to_string(),
                    }),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorMessage {
                        error: "Erro interno no servidor ao tentar login.".to_string(),
                    }),
                ))
            }
        }
    }
}
