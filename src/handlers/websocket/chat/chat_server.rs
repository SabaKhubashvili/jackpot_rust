use std::collections::HashMap;

use actix::{Actor, Context, Handler, Message, Recipient};
use serde::Serialize;

pub struct ChatServer{
    pub sessions: HashMap<i32,Recipient<ClientMessage>>
}
impl ChatServer{
    pub fn new()->Self{
        ChatServer{sessions:HashMap::new()}
    }
}

impl Actor for ChatServer{
    type Context = Context<Self>;
}


#[derive(Message)]
#[rtype (result = "()")]
pub struct Connect{
    pub user_id:i32,
    pub addr: Recipient<ClientMessage>
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

#[derive(Message,Clone,Serialize)]
#[rtype(result = "()")]
pub struct ClientMessage{
    pub sender_id:i32,
    pub message:String
}


impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.insert(msg.user_id, msg.addr);
        println!("User {} connected to server. Total sessions: {}", msg.user_id, self.sessions.len());
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        println!("Broadcasting message from user {}: {}", msg.sender_id, msg.message);
        for (id, addr) in &self.sessions {
            println!("Sending to user {}", id);
            addr.do_send(msg.clone());
        }
    }
}
