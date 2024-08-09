use actix::{clock::Instant, Addr};
use actix_web::{web::{Data, Payload}, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use coinflip_server::CoinflipServer;
use coinflip_ws::CoinflipWs;
use rand::Rng;

pub mod coinflip_server;
pub mod coinflip_ws;


pub async fn handle_coinflip_ws(
    stream: Payload,
    server: Data<Addr<CoinflipServer>>,
    req: HttpRequest
) -> impl Responder{
    let name = uuid::Uuid::new_v4().to_string();
    let user_id = rand::thread_rng().gen_range(0..2001);
    let res = ws::start(CoinflipWs{
        addr: server.get_ref().clone(),
        hb: Instant::now(),
        session_id: req.match_info().query("session_id").to_string(),
        name:Some(name),
        user_id
    }, &req, stream);
    match res{
        Ok(response) => response,
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}