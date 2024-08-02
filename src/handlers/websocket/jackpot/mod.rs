use std::time::Instant;

use actix::Addr;
use actix_web::{
    web::{Data, Payload},
    HttpRequest, HttpResponse, Responder,
};
use actix_web_actors::ws;
use jackpot_server::JackpotServer;
use jackpot_ws::JackpotWs;
use rand::Rng;

use crate::{db_utils::AppState, jwt::decode_jwt};

pub mod jackpot_server;
pub mod jackpot_ws;

pub async fn handle_jackpot_ws(
    jackpot_serv: Data<Addr<JackpotServer>>,
    app_state: Data<AppState>,
    req: HttpRequest,
    stream: Payload,
) -> impl Responder {
    let token = req
        .query_string()
        .split('&')
        .find(|&param| param.starts_with("token="))
        .and_then(|param| param.split('=').nth(1));

    match token {
        Some(tkn) => match decode_jwt(&tkn) {
            Ok(claims) => {
                let res = ws::start(
                    JackpotWs {
                        addr: jackpot_serv.get_ref().clone(),
                        name: Some(claims.claims.username),
                        hb: Instant::now(),
                        user_id: claims.claims.sub,
                        db_pool: Some(app_state.as_ref().db.clone()),
                    },
                    &req,
                    stream,
                );

                match res {
                    Ok(response) => response,
                    Err(e) => {
                        println!("{:?}", e);
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
            Err(_) => {
                println!("Couldnot decode jwt");
                HttpResponse::Unauthorized().finish()
            }
        },
        None => {
            let res = ws::start(
                JackpotWs {
                    addr: jackpot_serv.get_ref().clone(),
                    name: None,
                    hb: Instant::now(),
                    user_id: rand::thread_rng().gen_range(1..20000),
                    db_pool: None,
                },
                &req,
                stream,
            );
            match res {
                Ok(response) => response,
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
    }
}
