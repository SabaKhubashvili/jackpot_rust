
use std::time::Instant;

use actix::Addr;
use actix_web::{web::{Data, Payload}, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use chat_server::ChatServer;
use chat_ws::ChatWs;
use rand::Rng;

pub mod chat_ws;
pub mod chat_server;

pub async fn handle_chat_ws(
    chat_server: Data<Addr<ChatServer>>,
    req: HttpRequest,
    stream: Payload,
) -> impl Responder {
    let rand = rand::thread_rng().gen_range(1..1000);
    let res = ws::start(
        ChatWs {
            user_id: rand,
            hb: Instant::now(),
            addr: chat_server.get_ref().clone(),
        },
        &req,
        stream,
    );
    match res {
        Ok(response) => response,
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
