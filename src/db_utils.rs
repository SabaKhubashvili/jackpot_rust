use actix::{Actor, Addr, SyncContext};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

pub type AppDbType = Pool<ConnectionManager<PgConnection>>;

pub struct AppState {
    pub db: Addr<DbActor>,
}

pub struct DbActor(pub AppDbType);

impl Actor for DbActor {
    type Context = SyncContext<Self>;
}

pub fn get_db_pool(url: &str) -> AppDbType {
    let manager = ConnectionManager::new(url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create database pool");
    pool
}
