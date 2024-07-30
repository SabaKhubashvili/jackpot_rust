use actix_web::web::{self, Data, ServiceConfig};

use crate::handlers::websocket::chat::{chat_server::ChatServer, handle_chat_ws};
use actix::Actor;

pub fn init_websocket_routes(cfg: &mut ServiceConfig) {
  let chat_server = ChatServer::new().start();
  cfg.app_data(Data::new(chat_server)).service(
    web::scope("/ws")
      .service(web::resource("/chat").route(web::get().to(handle_chat_ws)))
  );
}
