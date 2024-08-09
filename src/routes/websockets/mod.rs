use actix_web::web::{self, ServiceConfig};

use crate::handlers::websocket::{
    chat::handle_chat_ws, coinflip::handle_coinflip_ws, crash::handle_crash_ws, jackpot::handle_jackpot_ws
};

pub fn init_websocket_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/ws")
            .service(web::resource("/chat").route(web::get().to(handle_chat_ws)))
            .service(web::resource("/jackpot").route(web::get().to(handle_jackpot_ws)))
            .service(web::resource("/crash").route(web::get().to(handle_crash_ws)))
            .service(web::resource("/coinflip").route(web::get().to(handle_coinflip_ws)))
    );
}
