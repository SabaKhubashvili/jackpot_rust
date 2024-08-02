use std::time::Instant;

use actix::Addr;
use actix_web::{
    web::{Data, Payload},
    HttpRequest, HttpResponse, Responder,
};
use actix_web_actors::ws;
use crash_server::CrashServer;
use crash_ws::CrashWs;
use rand::Rng;

pub mod crash_server;
pub mod crash_ws;

pub async fn handle_crash_ws(
    req: HttpRequest,
    serv: Data<Addr<CrashServer>>,
    stream: Payload,
) -> impl Responder {
    let res = ws::start(
        CrashWs {
            user_id: rand::thread_rng().gen_range(1..5000),
            hb: Instant::now(),
            addr: serv.get_ref().clone(),
        },
        &req,
        stream,
    );
    match res {
        Ok(response) => response,
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
