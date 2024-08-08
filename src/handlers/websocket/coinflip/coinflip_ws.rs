use std::time::Duration;

use actix::ActorContext;
use actix::Addr;
use actix::AsyncContext;
use actix::Handler;
use actix::StreamHandler;
use actix::{clock::Instant, Actor};
use actix_web_actors::ws;
use serde::Deserialize;


use super::coinflip_server::JoinGame;
use super::coinflip_server::Player;
use super::coinflip_server::{CoinflipServer,Connect,ClientMessage,Disconnect};

pub struct CoinflipWs {
    pub session_id: String,
    pub addr: Addr<CoinflipServer>,
    pub hb: Instant,
    pub user_id: i32,
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct RequestPayload{
    msg_type: String,
    payload: serde_json::Value,
}

#[derive(Deserialize)]
pub struct JoinPayload{
    user_id:usize,
    game_id:String
}


impl Actor for CoinflipWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        self.addr.do_send(Connect{
            user_id: self.user_id,
            addr: ctx.address().recipient(),
            session_id: self.session_id.clone()
        })
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.addr.do_send(Disconnect{
            user_id:self.user_id,
            session_id: self.session_id.clone()
        })
    }
}

impl StreamHandler<Result<ws::Message,ws::ProtocolError>> for CoinflipWs{
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg{
            Ok(ws::Message::Text(txt))=>{
                if let Ok(msg_type) = serde_json::from_str::<RequestPayload>(&txt){
                    match msg_type.msg_type.as_str(){
                        "join"=>{
                            let payload = serde_json::from_value::<JoinPayload>(msg_type.payload);
                            match payload{
                                Ok(user) => {
                                if let Some(name) = self.name.clone(){
                                    let new_player = Player{
                                        id:user.user_id,
                                        _name: name,
                                        addr: ctx.address().recipient()
                                    };
                                    self.addr.do_send(JoinGame{
                                        player:new_player,
                                        gameid:user.game_id.clone()
                                    })
                                }
                                },
                                Err(_) => {
                                    println!("Failed to deserialize JoinPayload");
                                    return;
                                }
 
                            }
                        },
                        _=>()
                    }
                }
            },
            Ok(ws::Message::Ping(m))=>{
                self.hb = Instant::now();
                ctx.pong(&m);
            },
            
            Ok(ws::Message::Pong(_))=>{
                self.hb = Instant::now();
            },
            Err(e)=>{
                println!("websocket error: {:?}", e);
                ctx.stop();
                return;
            },
            _=>()
        }
    }
}

impl CoinflipWs {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
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


impl Handler<ClientMessage> for CoinflipWs{
    type Result = ();
    fn handle(&mut self, msg: ClientMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg{
            ClientMessage::Text(txt)=>{
                ctx.text(txt);
            }, 
            ClientMessage::_Json(jsn)=>{
                if let Ok(json_msg) = serde_json::to_string(&jsn){
                    ctx.text(json_msg);
                }
            }
        }
    }
}