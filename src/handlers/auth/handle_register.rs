use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{db_utils::AppState, errors::auth::RegisterError, messages::auth::RegisterMessage};
#[derive(Serialize)]
struct RegisterRouteError {
    message: String,
    status: i32,
    variant: String,
}
#[derive(Deserialize)]
pub struct RegisterPayload {
    username: String,
    password: String,
}

pub async fn handle_register(
    payload: Json<RegisterPayload>,
    app_state: Data<AppState>,
) -> impl Responder {
    let conn = app_state.as_ref().db.clone();

    let result = conn
        .send(RegisterMessage {
            username: payload.username.clone(),
            password: payload.password.clone(),
        })
        .await;

    match result {
        Ok(Ok(user)) => HttpResponse::Created().json(json!({
            "success":true,
            "status":201,
            "message":"User succesfully registered",
            "user":{
                "username":user.username,
            }
        })),
        Ok(Err(err)) => match err {
            RegisterError::ForbiddenFormat => HttpResponse::BadRequest().json(RegisterRouteError {
                message: "Invalid format".to_string(),
                status: 400,
                variant: "ValidationError".to_string(),
            }),
            RegisterError::UsernameAlreadyRegistered => {
                HttpResponse::Conflict().json(RegisterRouteError {
                    message: "Username already registered".to_string(),
                    status: 409,
                    variant: "FieldTaken".to_string(),
                })
            }
            RegisterError::DieselError(e) => {
                HttpResponse::InternalServerError().json(RegisterRouteError {
                    message: format!("Database error: {}", e),
                    status: 500,
                    variant: "InternalServerError".to_string(),
                })
            }
            _ => HttpResponse::InternalServerError().json(RegisterRouteError {
                message: "Something went wrong".to_string(),
                status: 500,
                variant: "InternalServerError".to_string(),
            }),
        },
        Err(_) => HttpResponse::InternalServerError().json(RegisterRouteError {
            message: "Something went wrong".to_string(),
            status: 500,
            variant: "InternalServerError".to_string(),
        }),
    }
}
