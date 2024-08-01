use actix_web::web::{self, ServiceConfig};

use crate::handlers::websocket::{chat::handle_chat_ws, jackpot::handle_jackpot_ws};

pub fn init_websocket_routes(cfg: &mut ServiceConfig) {

  cfg.service(
    web::scope("/ws")
      .service(web::resource("/chat").route(web::get().to(handle_chat_ws)))
      .service(web::resource("/jackpot").route(web::get().to(handle_jackpot_ws)))
  );
}
