use std::time::{Duration, Instant};

use actix::StreamHandler;
use actix::{Actor, Addr, Handler};
use actix_web_actors::ws;

use super::crash_server::{ClientMessage, Connect, CrashServer, Disconnect};
use actix::ActorContext;
use actix::AsyncContext;

pub struct CrashWs {
    pub user_id: i32,
    pub hb: Instant,
    pub addr: Addr<CrashServer>,
}

impl Actor for CrashWs {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        self.addr.do_send(Connect {
            user_id: self.user_id,
            addr: ctx.address().recipient(),
        });
        println!("user with id: {} Connected", self.user_id);
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.addr.do_send(Disconnect {
            user_id: self.user_id,
        });
        println!("user with id: {} Disconnected", self.user_id);
    }
}

impl CrashWs {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        if Instant::now().duration_since(self.hb) >= Duration::from_secs(10) {
            ctx.stop();
            return;
        } else {
            ctx.ping(b"");
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for CrashWs {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg)
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(msg)) => {}
            Err(e) => {
                eprintln!("Error during WebSocket message: {}", e);
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl Handler<ClientMessage> for CrashWs {
    type Result = ();
    fn handle(&mut self, msg: ClientMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg.clone() {
            ClientMessage::Text(txt) => {
                ctx.text(txt);
            }
            ClientMessage::Json(val) => {
                let json_string = serde_json::to_string(&val);
                match json_string {
                    Ok(json_str) => {
                        ctx.text(json_str);
                    }
                    Err(e) => {
                        eprintln!("Error converting JSON: {}", e);
                    }
                }
            }
        }
    }
}
