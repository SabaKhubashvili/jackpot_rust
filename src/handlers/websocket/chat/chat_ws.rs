use std::time::Duration;

use actix::{Handler, StreamHandler};
use actix::{clock::Instant, Actor, Addr};
use actix_web_actors::ws;

use super::chat_server::{ChatServer, ClientMessage, Connect};
use actix::AsyncContext;
use actix::ActorContext;

pub struct ChatWs{
    pub user_id:i32,
    pub addr: Addr<ChatServer>,
    pub hb: Instant
}


impl Actor for ChatWs{
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb = Instant::now();
        self.hb(ctx);
        self.addr.do_send(Connect{
            user_id:self.user_id.clone(),
            addr: ctx.address().recipient()
        })

    }
}
impl ChatWs {
    fn hb(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(Duration::from_secs(5), |act, ctx| {
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
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
                self.hb = Instant::now();
            },
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            },
            Ok(ws::Message::Text(msg)) => {
                println!("{:?}", msg);
                let msg = ClientMessage {
                    sender_id: self.user_id,
                    message: msg.to_string(),
                };
                self.addr.do_send(msg);
            },
            Err(err) => {
                eprintln!("Error: {}", err);
                ctx.stop();
            },
            _ => ()
        }
    }
}


impl Handler<ClientMessage> for ChatWs{
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, ctx: &mut Self::Context) -> Self::Result {
        if let Ok(serialized_msg) = serde_json::to_string(&msg){
            ctx.text(serialized_msg);
        }else{
            eprintln!("Failed to serialize message");
            return;
        }
    }
}