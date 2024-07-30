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
use actix::SyncArbiter;
use db_utils::{get_db_pool, AppState, DbActor};
use dotenv::dotenv;
use std::env;

use actix_web::{web::Data, App, HttpServer};
use routes::init_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL cannot be empty (env)");
    let pool = get_db_pool(&database_url);
    let db_addr = SyncArbiter::start(5, move || DbActor(pool.clone()));
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                db: db_addr.clone(),
            }))
            .configure(init_routes)
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
