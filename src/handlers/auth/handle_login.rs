use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    db_utils::AppState, errors::auth::LoginError, jwt::generate_jwt, messages::auth::LoginMessage,
};

#[derive(Deserialize)]
pub struct LoginPayload {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginRouteError {
    message: String,
    status: i32,
    variant: String,
}

pub async fn handle_login(
    payload: Json<LoginPayload>,
    app_state: Data<AppState>,
) -> impl Responder {
    let conn = app_state.as_ref().db.clone();

    let result = conn
        .send(LoginMessage {
            username: payload.username.clone(),
            password: payload.password.clone(),
        })
        .await;

    match result {
        Ok(Ok(user)) => {
            let expiration_date = Utc::now()
                .checked_add_signed(Duration::days(2))
                .expect("invalid timestamp")
                .timestamp();
            let token =
                generate_jwt(&user.username, user.id, expiration_date as usize).map_err(|_| {
                    HttpResponse::InternalServerError().json(LoginRouteError {
                        message: "Failed to generate token".to_string(),
                        status: 500,
                        variant: "InternalError".to_string(),
                    })
                });
            match token {
                Ok(token) => HttpResponse::Ok().json(json!({
                    "username": user.username,
                    "token": token
                })),
                Err(_) => HttpResponse::InternalServerError().json(LoginRouteError {
                    message: "Failed to generate token".to_string(),
                    status: 500,
                    variant: "InternalError".to_string(),
                }),
            }
        }
        Ok(Err(e)) => match e {
            LoginError::InvalidCredentials => HttpResponse::Unauthorized().json(LoginRouteError {
                message: "Invalid credentials".to_string(),
                status: 401,
                variant: "ValidationError".to_string(),
            }),
            _ => HttpResponse::InternalServerError().json(LoginRouteError {
                message: "Internal server error".to_string(),
                status: 500,
                variant: "InternalError".to_string(),
            }),
        },
        Err(_) => HttpResponse::InternalServerError().json(LoginRouteError {
            message: "Internal server error".to_string(),
            status: 500,
            variant: "InternalError".to_string(),
        }),
    }
}
