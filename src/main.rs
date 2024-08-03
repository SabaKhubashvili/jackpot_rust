mod actors;
mod db_utils;
mod errors;
mod handlers;
mod jwt;
mod messages;
mod middlewares;
mod models;
mod routes;
mod schema;
mod validation;
mod websockets;
use actix::Actor;
use actix::SyncArbiter;
use actix_cors::Cors;
use actix_web::{http::header, web::Data, App, HttpServer};
use db_utils::{get_db_pool, AppState, DbActor};
use dotenv::dotenv;
use handlers::websocket::{
    chat::chat_server::ChatServer, crash::crash_server::CrashServer,
    jackpot::jackpot_server::JackpotServer,
};
use routes::init_routes;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL cannot be empty (env)");
    let pool = get_db_pool(&database_url);
    let db_addr = SyncArbiter::start(5, move || DbActor(pool.clone()));
    let chat_server = ChatServer::new().start();
    let jackpot_server = JackpotServer::new().start();
    let crash_server = CrashServer::new().start();

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://127.0.0.1:5500")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![
                        header::AUTHORIZATION,
                        header::ACCEPT,
                        header::CONTENT_TYPE,
                    ])
                    .supports_credentials()
                    .max_age(3600),
            )
            .app_data(Data::new(jackpot_server.clone()))
            .app_data(Data::new(chat_server.clone()))
            .app_data(Data::new(crash_server.clone()))
            .app_data(Data::new(AppState {
                db: db_addr.clone(),
            }))
            .configure(init_routes)
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
