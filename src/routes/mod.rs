pub mod auth;
pub mod websockets;

use actix_web::web::ServiceConfig;
use auth::init_auth_routes;
use websockets::init_websocket_routes;

pub fn init_routes(cfg: &mut ServiceConfig) {
    cfg.configure(init_auth_routes)
        .configure(init_websocket_routes);
}
