use std::time::{Duration, Instant};

use actix::{Actor, Addr, Handler, StreamHandler};
use actix_web_actors::ws;
use serde::Deserialize;

use crate::db_utils::DbActor;

use super::jackpot_server::{ClientMessage, Connect, Deposit, Disconnect, JackpotServer, Player};
use actix::AsyncContext;

pub struct JackpotWs {
    pub addr: Addr<JackpotServer>,
    pub hb: Instant,
    pub user_id: i32,
    pub name: Option<String>,
    pub db_pool: Option<Addr<DbActor>>,
}
#[derive(Deserialize, Debug)]
pub struct DepositPayload {
    pub amount: f64,
}

impl Actor for JackpotWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!(
            "User with id {} started, Authorized: {}",
            self.user_id,
            self.name.is_some()
        );
        self.hb(ctx);
        self.addr.do_send(Connect {
            user_id: self.user_id.clone(),
            addr: ctx.address().recipient(),
        })
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.addr.do_send(Disconnect {
            user_id: self.user_id.clone(),
        })
    }
}

impl JackpotWs {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        if Instant::now().duration_since(self.hb) > Duration::from_secs(15) {
            ctx.close(None);
            return;
        } else {
            ctx.ping(b"");
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for JackpotWs {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg)
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(msg)) => {
                let deserialized = serde_json::from_str::<DepositPayload>(&msg);
                if let Some(name) = &self.name {
                    if let Some(db_pool) = &self.db_pool {
                        match deserialized {
                            Ok(msg) => {
                                let deposit = Deposit {
                                    player: Player {
                                        user_id: self.user_id,
                                        name: name.to_string(),
                                        deposit: msg.amount,
                                    },
                                    db_pool: db_pool.clone(),
                                };
                                self.addr.do_send(deposit);
                            }
                            Err(_) => {
                                println!("Closing error: {}", msg);
                                self.addr.do_send(ClientMessage {
                                    msg: "Invalid payload".to_string(),
                                    variant: "error".to_string(),
                                });
                                // ctx.close(None);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                ctx.close(None);
            }
            _ => (),
        }
    }
}

impl Handler<ClientMessage> for JackpotWs {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, ctx: &mut Self::Context) -> Self::Result {
        if let Ok(json_msg) = serde_json::to_string(&msg) {
            ctx.text(json_msg);
        } else {
            eprintln!("Failed to serialize");
            return;
        }
    }
}
