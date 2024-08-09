use std::collections::HashMap;

use actix::{Actor, AsyncContext, Context, Handler, Message, Recipient};
use rand::Rng;
use serde::Serialize;
use serde_json::json;


//* --- Struct --- */
pub struct CoinflipServer {
    pub spectators: Vec<i32>,
    pub sessions: HashMap<String, CoinflipGame>,
}
impl CoinflipServer{
    pub fn new() -> Self{
        CoinflipServer{
            spectators: Vec::new(),
            sessions: HashMap::new(),
        }
    }
}
#[derive(Debug)]
pub struct CoinflipGame {
    pub spectators: HashMap<i32,Recipient<ClientMessage>>,
    pub players: Vec<Player>,
    pub amount: f64,
}
#[derive(Clone,Debug)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub addr: Recipient<ClientMessage>,
}
#[derive(Serialize)]
pub struct JsonResponse {
    pub message_type: String,
    pub payload: serde_json::Value,
}
//* --- X --- */
//* --- Messages --- */
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub session_id: String,
    pub user_id: i32,
    pub addr: Recipient<ClientMessage>,
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub session_id: String,
    pub user_id: i32
}
#[derive(Message)]
#[rtype(result = "()")]
pub enum ClientMessage {
    Text(String),
    Json(JsonResponse),
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct AddGame {
    pub amount: f64,
    pub player: Player,
}

#[derive(Message,Debug)]
#[rtype(result = "()")]
pub struct JoinGame {
    pub gameid: String,
    pub player: Player,
}
//* X */
//* --- Actor --- */
impl Actor for CoinflipServer {
    type Context = Context<Self>;
}
impl Actor for CoinflipGame {
    type Context = Context<Self>;
}

//* --- X --- */
//* --- Handler --- */
impl Handler<Connect> for CoinflipServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        self.spectators.push(msg.user_id);
    }
}
impl Handler<Disconnect> for CoinflipServer {
    type Result = ();
    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.spectators.retain(|p| *p != msg.user_id);
    }
}
impl Handler<ClientMessage> for CoinflipGame {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ClientMessage::Text(val) => {
                for player in &self.players {
                    player.addr.do_send(ClientMessage::Text(val.clone()));
                }
                for spectator in self.spectators.values() {
                    spectator.do_send(ClientMessage::Text(val.clone()));
                }
            }
            ClientMessage::Json(val) => {
                let json_val = serde_json::to_string(&val).unwrap();
                for player in &self.players {
                    player.addr.do_send(ClientMessage::Text(json_val.clone()));
                }
                for spectator in self.spectators.values() {
                    spectator.do_send(ClientMessage::Text(json_val.clone()));
                }
            }
        }
    }
}
impl Handler<ClientMessage> for CoinflipServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ClientMessage::Text(val) => {
                for session in self.sessions.values() {
                    for player in &session.players {
                        player.addr.do_send(ClientMessage::Text(val.clone()));
                    }
                    for spectator in session.spectators.values() {
                        spectator.do_send(ClientMessage::Text(val.clone()));
                    }
                }
            }
            ClientMessage::Json(val) => {
                let json_val = serde_json::to_string(&val).unwrap();
                for session in self.sessions.values() {
                    for player in &session.players {
                        player.addr.do_send(ClientMessage::Text(json_val.clone()));
                    }
                    for spectator in session.spectators.values() {
                        spectator.do_send(ClientMessage::Text(json_val.clone()));
                    }
                }
            }
        }
    }
}
impl Handler<AddGame> for CoinflipServer {
    type Result = ();
    fn handle(&mut self, msg: AddGame, ctx: &mut Self::Context) -> Self::Result {
        let id = uuid::Uuid::new_v4().to_string();
        let new_game = CoinflipGame::new(msg.amount, msg.player.clone());
        self.sessions.insert(id.clone(), new_game);
        println!("{:?}",self.sessions);
        ctx.address().do_send(ClientMessage::Json(JsonResponse{
            message_type:String::from("new_game"),
            payload: json!({
                "game_id": id,
                "amount": msg.amount,
                "player": {
                    "id": msg.player.id,
                    "name": msg.player.name,
                }
            })
        }));
    }
}

impl Handler<JoinGame> for CoinflipServer {
    type Result = ();

    fn handle(&mut self, msg: JoinGame, _ctx: &mut Self::Context) -> Self::Result {
        println!("{:?}",msg);
        println!("{:?}",self.sessions);
        if let Some(game) = self.sessions.get_mut(&msg.gameid) {
            println!("{:?}",game);
            if game.players.iter().all(|p| p.id != msg.player.id) {
                game.players.insert(msg.player.id, msg.player);
                println!("user joined");
                if game.players.len() == 2 {
                    game.start_game();
                }
            }
        }
    }
}

//* --- X --- */
//* Implementations */
impl CoinflipGame {
    fn new(amount: f64, player: Player) -> Self {
        let rounded_amount = round_to(amount, 2);
        CoinflipGame {
            spectators: HashMap::new(),
            players: vec![player],
            amount: rounded_amount,
        }
    }
    fn start_game(&mut self) {
        let rand_num = rand::thread_rng().gen_range(0..2);

        let winner = &self.players[rand_num];
        let loser = &self.players[1 - rand_num];

        let winner_msg = format!("You won ${:.2}!", self.amount);
        let loser_msg = "You lost. Better luck next time!".to_string();
        let game_result_msg = format!("Player {} won ${:.2}!", winner.name, self.amount);

        // Send messages to players
        winner.addr.do_send(ClientMessage::Text(winner_msg.clone()));
        loser.addr.do_send(ClientMessage::Text(loser_msg.clone()));

        // Broadcast the result to all spectators and players
        for spectator in self.spectators.values() {
            spectator.do_send(ClientMessage::Text(game_result_msg.clone()));
        }
    }
}

fn round_to(value: f64, decimal_places: u32) -> f64 {
    let factor = 10f64.powi(decimal_places as i32);
    (value * factor).round() / factor
}
