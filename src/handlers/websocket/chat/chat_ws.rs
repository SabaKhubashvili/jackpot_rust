use std::time::{Duration, Instant};

use actix::Addr;
use actix_web_actors::ws;


use super::chat_server::{ChatServer, ClientMessage, Connect, Disconnect};
use actix::prelude::*;
pub struct ChatWs {
    pub user_id: i32,
    pub hb: Instant,
    pub addr: Addr<ChatServer>,
}

impl Actor for ChatWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        self.addr.do_send(Connect {
            id: self.user_id,
            addr: ctx.address().recipient(),
        })
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.addr.do_send(Disconnect { id: self.user_id })
    }
}

impl ChatWs {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(Duration::new(5, 0), |act, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::from_secs(10) {
                ctx.stop();
                return;
            } else {
                ctx.ping(b"");
            }
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatWs {
    fn handle(&mut self, message: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match message {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(msg)) => {
                let payload = ClientMessage {
                    id: self.user_id,
                    msg: msg.to_string()
                };
                self.addr.do_send(payload);
            }
            Err(err) => {
                eprintln!("WebSocket error: {:?}", err);
                ctx.stop();
            }
            _ => (),
        };
    }
}

impl Handler<ClientMessage> for ChatWs {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, ctx: &mut Self::Context) {
        if let Ok(json_msg) = serde_json::to_string(&msg) {
            ctx.text(json_msg);
        } else {
            eprintln!("Failed to serialize message to JSON");
        }
    }
}






