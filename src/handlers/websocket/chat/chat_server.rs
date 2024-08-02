use actix::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
pub struct ChatServer {
    sessions: HashMap<i32, Recipient<ClientMessage>>,
}

impl ChatServer {
    pub fn new() -> Self {
        ChatServer {
            sessions: HashMap::new(),
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: i32,
    pub addr: Recipient<ClientMessage>,
}

impl Handler<Connect> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.insert(msg.id, msg.addr);
    }
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: i32,
}
impl Handler<Disconnect> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.remove(&msg.id);
    }
}

#[derive(Message, Clone, Serialize)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: i32,
    pub msg: String,
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        for (_id, addr) in &self.sessions {
            addr.do_send(msg.clone());
        }
    }
}
