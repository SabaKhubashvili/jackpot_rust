use std::{collections::HashMap, time::Duration};

use actix::{Actor, AsyncContext, Context, Handler, Message, Recipient};

use crate::handlers::websocket::chat::chat_server::ClientMessage;



pub struct CrashServer{
    pub sessions: HashMap<i32,Recipient<ClientMessage>>,
    pub crash_game: CrashGame
}

impl Actor for CrashServer{
    type Context = Context<Self>;

    // fn started(&mut self, ctx: &mut Self::Context) {
        
    // }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub user_id:i32,
    pub addr: Recipient<ClientMessage>,
}

impl Handler<Connect> for CrashServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result {
        self.sessions.insert(msg.user_id,msg.addr);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub user_id:i32
}

impl Handler<Disconnect> for CrashServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result {
        self.sessions.remove(&msg.user_id);
    }
}


pub struct Player{
    cashed_out:bool,
    deposit: i32,
    user_id:i32
}

pub struct CrashGame{
    pub is_started:bool,
    pub players: Vec<Player>,
}

impl Actor for CrashGame{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        while true {
            if !self.is_started{
                ctx.run_interval(Duration::from_millis(millis), f)
            }
        }
    }
}