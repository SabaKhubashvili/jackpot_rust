use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, Recipient};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

use crate::db_utils::DbActor;

pub struct JackpotServer {
    pub sessions: HashMap<i32, Recipient<ClientMessage>>,
    pub game_session: Option<GameSession>,
}

impl JackpotServer {
    pub fn new() -> Self {
        JackpotServer {
            sessions: HashMap::new(),
            game_session: None,
        }
    }

    fn notify_timer_start(&self, _ctx: &mut Context<Self>) {
        let timer_start_message = ClientMessage {
            msg: "The game timer has started. 15 seconds until the winner is chosen.".into(),
            variant: "timer_start".into(),
        };

        for addr in self.sessions.values() {
            addr.do_send(timer_start_message.clone());
        }
    }

    pub fn notify_winner(&self, player: Player) {
        let full_amount = self
            .game_session
            .as_ref()
            .map(|session| {
                session
                    .players
                    .values()
                    .map(|player| player.deposit)
                    .sum::<f64>()
            })
            .unwrap_or(0.0);

        let winner_message = ClientMessage {
            msg: format!("{} has won the jackpot of {}", player.name, full_amount),
            variant: "winner".into(),
        };

        for addr in self.sessions.values() {
            addr.do_send(winner_message.clone());
        }
    }

    pub fn notify_player_join(&self, player: &Player) {
        let player_join_message = ClientMessage {
            msg: format!(
                "{} has joined the game with a deposit of {}",
                player.name, player.deposit
            ),
            variant: "player_join".into(),
        };

        for addr in self.sessions.values() {
            addr.do_send(player_join_message.clone());
        }
    }

    pub fn reset_game(&mut self) {
        self.game_session = None;
        let game_reset_message = ClientMessage {
            msg: "Resetting game!".into(),
            variant: "reset".into(),
        };

        for addr in self.sessions.values() {
            addr.do_send(game_reset_message.clone());
        }
    }
}

impl Actor for JackpotServer {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Deposit {
    pub player: Player,
    pub db_pool: Addr<DbActor>,
}

impl Handler<Deposit> for JackpotServer {
    type Result = ();

    fn handle(&mut self, msg: Deposit, ctx: &mut Self::Context) -> Self::Result {
        if let Some(ref mut session) = self.game_session {
            session.add_player(msg.player.clone());
        } else {
            let mut new_session = GameSession::new();
            new_session.add_player(msg.player.clone());
            self.game_session = Some(new_session);
        }

        self.notify_player_join(&msg.player);

        msg.db_pool.send(RecordDeposit {
            user_id: msg.player.user_id,
            amount: msg.player.deposit,
        });

        if let Some(ref mut session) = self.game_session {
            if session.players.len() >= 2 && !session.timer_started {
                session.timer_started = true;
                self.notify_timer_start(ctx);
                ctx.run_later(Duration::from_secs(15), |act, _ctx| {
                    if let Some(ref mut session) = act.game_session {
                        if let Some(winner) = session.start_game() {
                            act.notify_winner(winner);
                            act.reset_game();
                        }
                    }
                });
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub user_id: i32,
    pub addr: Recipient<ClientMessage>,
}

impl Handler<Connect> for JackpotServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.insert(msg.user_id, msg.addr);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub user_id: i32,
}

impl Handler<Disconnect> for JackpotServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.remove(&msg.user_id);
    }
}

#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub msg: String,
    pub variant: String,
}

impl Handler<ClientMessage> for JackpotServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        for addr in self.sessions.values() {
            addr.do_send(msg.clone());
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub user_id: i32,
    pub name: String,
    pub deposit: f64,
}

#[derive(Clone, Debug)]
pub struct GameSession {
    pub players: HashMap<i32, Player>,
    pub timer_started: bool,
}

impl GameSession {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            timer_started: false,
        }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.user_id, player);
    }

    pub fn start_game(&self) -> Option<Player> {
        let full_amount: f64 = self.players.values().map(|x| x.deposit).sum();
        let mut rand = rand::thread_rng();
        let mut roll = rand.gen_range(0.0..full_amount);

        for player in self.players.values() {
            if roll <= player.deposit {
                return Some(player.clone());
            } else {
                roll -= player.deposit;
            }
        }
        None
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RecordDeposit {
    pub user_id: i32,
    pub amount: f64,
}

impl Handler<RecordDeposit> for DbActor {
    type Result = ();

    fn handle(&mut self, msg: RecordDeposit, ctx: &mut Self::Context) -> Self::Result {
        let conn = self.0.get().expect("Failed to connect");
    }
}
