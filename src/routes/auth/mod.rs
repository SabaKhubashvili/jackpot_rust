use actix_web::web::{self, post, ServiceConfig};

use crate::handlers::auth::{handle_login::handle_login, handle_register::handle_register};

pub fn init_auth_routes(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("/register").route(post().to(handle_register)))
        .service(web::resource("/login").route(post().to(handle_login)));
}
