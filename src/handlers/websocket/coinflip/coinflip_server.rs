use std::collections::HashMap;

use actix::{Actor, Context, Handler, Message, Recipient};
use rand::Rng;
use serde::Serialize;


//* --- Struct --- */

pub struct CoinflipServer{
    pub sessions: HashMap<String,CoinflipGame>
}


pub struct CoinflipGame{
    pub spectators: Vec<Recipient<ClientMessage>>,
    pub players: Vec<Player>,
    pub amount: f64
}
#[derive(Clone)]
pub struct Player{
    pub id: usize,
    pub _name: String,
    pub addr: Recipient<ClientMessage>,
}
#[derive(Serialize)]
pub struct JsonResponse {
    pub message_type: String,
    pub payload: serde_json::Value
}
//* --- X --- */

//* --- Messages --- */
#[derive(Message)]
#[rtype(result="()")]
pub enum ClientMessage{
    Text(String),
    _Json(JsonResponse)
}

#[derive(Message)]
#[rtype(result="()")]
pub struct AddGame{
    pub amount: f64,
    pub player: Player
}

#[derive(Message)]
#[rtype(result="()")]
pub struct JoinGame{
    pub gameid: String,
    pub player: Player,
}
//* X */
//* --- Actor --- */
impl Actor for CoinflipServer{
    type Context = Context<Self>;
}
impl Actor for CoinflipGame{
    type Context = Context<Self>;
}

//* --- X --- */


//* --- Handler --- */
impl Handler<ClientMessage> for CoinflipGame{
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg{
            ClientMessage::Text(val)=>{
                for addr in self.sessions.values(){
                    addr.do_send(ClientMessage::Text(val.clone()))
                }
            },
            ClientMessage::_Json(val)=>{
                for addr in self.sessions.values(){
                    if let Ok(json_val) =  serde_json::to_string(&val){
                        addr.do_send(ClientMessage::Text(json_val.clone()))
                    }
                }
            }
        }
    }
}

impl Handler<AddGame> for CoinflipServer{
    type Result = ();
    fn handle(&mut self, msg: AddGame, _ctx: &mut Self::Context) -> Self::Result {
        let id = uuid::Uuid::new_v4().to_string();
        let new_game = CoinflipGame::new(msg.amount,msg.player);
        self.sessions.insert(id, new_game);
    }
}

impl Handler<JoinGame> for CoinflipServer{
    type Result = ();

    fn handle(&mut self, msg: JoinGame, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(game) = self.sessions.get_mut(&msg.gameid){
            if game.players.iter().all(|p| p.id != msg.player.id) {
                game.players.insert(msg.player.id,msg.player);

                if game.players.len() == 2{
                    game.start_game();
                }
            }
        }
    }
}

//* --- X --- */

//* Implementations */

impl CoinflipGame{
    fn new(amount: f64, player: Player) -> Self{
        let rounded_amount = round_to(amount,2);
        CoinflipGame{
            spectators: vec![],
            players: vec![player],
            amount: rounded_amount
        }
    }
    fn start_game(&mut self){
        let rand_num = rand::thread_rng().gen_range(0..2);

        let winner = &self.players[rand_num];
        let loser = &self.players[1 - rand_num];

        let winner_msg = format!("You won ${:.2}!", self.amount);
        let loser_msg = "You lost. Better luck next time!".to_string();
        let game_result_msg = format!("Player {} won ${:.2}!", winner._name, self.amount);

        // Send messages to players
        winner.addr.do_send(ClientMessage::Text(winner_msg.clone()));
        loser.addr.do_send(ClientMessage::Text(loser_msg.clone()));

        // Broadcast the result to all spectators and players
        for spectator in &self.spectators {
            spectator.do_send(ClientMessage::Text(game_result_msg.clone()));
        }
    }
}




fn round_to(value:f64,decimal_places: u32) -> f64{
    let factor = 10f64.powi(decimal_places as i32);
    (value * factor).round() / factor
}