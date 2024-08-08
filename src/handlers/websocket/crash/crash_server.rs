use std::{collections::HashMap, sync::Arc, time::Duration, u32};

use actix::ActorFutureExt;
use actix::{
    clock::Instant, Actor, Addr, AsyncContext, Context, Handler, Message, Recipient, WrapFuture,
};
use rand::{rngs::OsRng, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;
pub struct CrashServer {
    pub sessions: HashMap<i32, Recipient<ClientMessage>>,
    pub crash_game: Option<Addr<CrashGame>>,
}

impl Actor for CrashServer {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Starting crash");
        let crash_game = CrashGame::new(_ctx.address()).start();
        self.crash_game = Some(crash_game);
    }
}

impl CrashServer {
    pub fn new() -> Self {
        CrashServer {
            sessions: HashMap::new(),
            crash_game: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientMessageJson {
    pub msg: String,
    pub msg_type: String,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub enum ClientMessage {
    Text(String),
    Json(ClientMessageJson),
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub user_id: i32,
    pub addr: Recipient<ClientMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub user_id: i32,
}
#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct DepositInCrash {
    pub user_id: i32,
    pub amount: f64,
}
#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct CashOut {
    pub user_id: i32,
}

impl Handler<DepositInCrash> for CrashServer {
    type Result = ();

    fn handle(&mut self, msg: DepositInCrash, ctx: &mut Self::Context) -> Self::Result {
        if let Some(game) = &self.crash_game {
            let user_id = msg.user_id;
            let session_clone = self.sessions.clone();

            let result_future = game.send(AddPlayerToCrash {
                user_id: msg.user_id,
                bet_amount: round_to(msg.amount, 2),
            });

            ctx.spawn(result_future.into_actor(self).map(move |result, _act, _| {
                let json = match result {
                    Ok(Ok(_)) => ClientMessageJson {
                        msg_type: "success_deposit".to_string(),
                        msg: "Successfully deposited".to_string(),
                    },
                    Ok(Err(DepositInCrashError::GameAlreadyStarted)) => ClientMessageJson {
                        msg_type: "failed_to_deposit".to_string(),
                        msg: "Game already started".to_string(),
                    },
                    Err(_) => {
                        println!("Failed to deposit in crash");
                        ClientMessageJson {
                            msg_type: "failed_to_deposit".to_string(),
                            msg: "Failed to deposit into crash game; game is already started"
                                .to_string(),
                        }
                    }
                };

                if let Some(addr) = session_clone.get(&user_id) {
                    addr.do_send(ClientMessage::Json(json));
                }
            }));
        }
    }
}
impl Handler<CashOut> for CrashServer {
    type Result = ();

    fn handle(&mut self, msg: CashOut, ctx: &mut Self::Context) -> Self::Result {
        if let Some(game) = &self.crash_game {
            let clone = game.clone();
            println!("{:?}", clone);
            let session_clone = self.sessions.clone();
            ctx.spawn(
                async move {
                    let res = clone
                        .send(CashOutFromCrash {
                            user_id: msg.user_id,
                        })
                        .await;
                    match res {
                        Ok(Ok(_)) => {
                            println!("Cashed out from crash game");
                            let json = ClientMessageJson {
                                msg_type: "success_cashout".to_string(),
                                msg: "Cashed out successfully".to_string(),
                            };
                            if let Some(addr) = session_clone.get(&msg.user_id) {
                                addr.do_send(ClientMessage::Json(json));
                            }
                        }
                        Ok(Err(CashoutFromCrashError::AlreadyCashedOut)) => {
                            let json = ClientMessageJson {
                                msg_type: "failed_to_cashout".to_string(),
                                msg: "Already cashed out".to_string(),
                            };
                            if let Some(addr) = session_clone.get(&msg.user_id) {
                                addr.do_send(ClientMessage::Json(json));
                            }
                        }
                        Ok(Err(CashoutFromCrashError::GameNotStarted)) => {
                            let json = ClientMessageJson {
                                msg_type: "failed_to_cashout".to_string(),
                                msg: "Game not started".to_string(),
                            };
                            if let Some(addr) = session_clone.get(&msg.user_id) {
                                addr.do_send(ClientMessage::Json(json));
                            }
                        }
                        Ok(Err(CashoutFromCrashError::UserNotFound)) => {
                            let json = ClientMessageJson {
                                msg_type: "failed_to_cashout".to_string(),
                                msg: "User haven't deposited".to_string(),
                            };
                            if let Some(addr) = session_clone.get(&msg.user_id) {
                                addr.do_send(ClientMessage::Json(json));
                            }
                        }
                        Err(_) => {
                            let json = ClientMessageJson {
                                msg_type: "failed_to_cashout".to_string(),
                                msg: "Failed to cash out, Something went wrong".to_string(),
                            };
                            if let Some(addr) = session_clone.get(&msg.user_id) {
                                addr.do_send(ClientMessage::Json(json));
                            }
                        }
                    }
                }
                .into_actor(self),
            );
        }
    }
}
impl Handler<ClientMessage> for CrashServer {
    type Result = ();
    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ClientMessage::Text(text_msg) => {
                for addr in self.sessions.values() {
                    addr.do_send(ClientMessage::Text(text_msg.clone()));
                }
            }
            ClientMessage::Json(json_msg) => {
                println!("Received JSON message: {}", json_msg.msg);
                if let Ok(json_msg_str) = serde_json::to_string(&json_msg) {
                    for addr in self.sessions.values() {
                        addr.do_send(ClientMessage::Text(json_msg_str.clone()));
                    }
                } else {
                    println!("Failed to serialize JSON message");
                }
            }
        }
    }
}

impl Handler<Connect> for CrashServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.insert(msg.user_id, msg.addr);
    }
}

impl Handler<Disconnect> for CrashServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.remove(&msg.user_id);
    }
}

#[derive(Debug)]
pub struct Bet {
    pub user_id: i32,
    pub bet_amount: f64,
    pub cashed_out: bool,
}

pub struct CrashGame {
    game_started: bool,
    players: Vec<Bet>,
    multiplier: f64,
    crash_point: Option<f64>,
    crashed: bool,
    started_at: Option<Instant>,
    round_id: Option<String>,
    public_seed: Option<String>,
    private_seed: Option<String>,
    interval_active: bool,

    server_addr: Addr<CrashServer>,
}

#[derive(Message)]
#[rtype(result = "Result<(),DepositInCrashError>")]
pub struct AddPlayerToCrash {
    pub user_id: i32,
    pub bet_amount: f64,
}
#[derive(Message)]
#[rtype(result = "Result<(),CashoutFromCrashError>")]
pub struct CashOutFromCrash {
    pub user_id: i32,
}
#[derive(Error, Debug)]
pub enum CashoutFromCrashError {
    #[error("Game is already started")]
    GameNotStarted,
    #[error("User not found in the game")]
    UserNotFound,
    #[error("Already cashed out")]
    AlreadyCashedOut,
}

#[derive(Error, Debug)]
pub enum DepositInCrashError {
    #[error("Game is already started")]
    GameAlreadyStarted,
}

impl Handler<AddPlayerToCrash> for CrashGame {
    type Result = Result<(), DepositInCrashError>;
    fn handle(&mut self, msg: AddPlayerToCrash, _ctx: &mut Self::Context) -> Self::Result {
        if !self.interval_active {
            let new_player = Bet {
                user_id: msg.user_id,
                bet_amount: msg.bet_amount,
                cashed_out: false,
            };
            self.players.push(new_player);
            Ok(())
        } else {
            Err(DepositInCrashError::GameAlreadyStarted)
        }
    }
}
impl Handler<CashOutFromCrash> for CrashGame {
    type Result = Result<(), CashoutFromCrashError>;

    fn handle(&mut self, msg: CashOutFromCrash, _ctx: &mut Self::Context) -> Self::Result {
        if self.interval_active {
            println!("players: {:?}", self.players);
            println!("User id: {}", msg.user_id);

            if let Some(player) = self.players.iter_mut().find(|p| p.user_id == msg.user_id) {
                if !player.cashed_out {
                    player.cashed_out = true;
                    Ok(())
                } else {
                    Err(CashoutFromCrashError::AlreadyCashedOut)
                }
            } else {
                Err(CashoutFromCrashError::UserNotFound)
            }
        } else {
            Err(CashoutFromCrashError::GameNotStarted)
        }
    }
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct ResetCrashGame;

impl Handler<ResetCrashGame> for CrashGame {
    type Result = ();
    fn handle(&mut self, _msg: ResetCrashGame, ctx: &mut Self::Context) -> Self::Result {
        ctx.run_later(Duration::from_secs(15), |act, ctx| {
            act.reset_game();
            act.run_game_loop(ctx)
        });
    }
}
impl Actor for CrashGame {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Crash loop starting");
        self.game_started = true;
        self.run_game_loop(_ctx);
        println!("Crash loop started");
    }
}

impl CrashGame {
    fn new(addr: Addr<CrashServer>) -> Self {
        Self {
            game_started: false,
            players: Vec::new(),
            multiplier: 1.00,
            crashed: false,
            crash_point: None,
            started_at: None,
            round_id: None,
            public_seed: None,
            private_seed: None,
            interval_active: false,
            server_addr: addr,
        }
    }

    pub fn run_game_loop(&mut self, ctx: &mut Context<Self>) {
        let round_id = Uuid::new_v4().to_string();
        let public_seed: [u8; 32] = OsRng.gen();
        let private_seed: [u8; 32] = OsRng.gen();

        let public_seed_hex = hex::encode(public_seed);
        let private_seed_hex = hex::encode(private_seed);

        self.round_id = Some(round_id);
        self.public_seed = Some(public_seed_hex.clone());
        self.private_seed = Some(private_seed_hex.clone());

        let crash_point = self.gen_crash_point(&private_seed_hex, &private_seed_hex);
        self.crash_point = Some(crash_point);
        self.interval_active = true;
        self.started_at = Some(Instant::now());
        let json = ClientMessageJson {
            msg_type: "start".to_string(),
            msg: format!("Crash Started, public seed: {}", public_seed_hex),
        };
        self.server_addr.do_send(ClientMessage::Json(json));
        ctx.run_interval(Duration::from_millis(200), |act, ctx| {
            if act.interval_active {
                act.update_game(ctx);
                if act.crashed {
                    let addr = ctx.address();
                    let json = ClientMessageJson {
                        msg_type: "crash".to_string(),
                        msg: format!("Crash point reached: {}", act.crash_point.unwrap()),
                    };
                    act.server_addr.do_send(ClientMessage::Json(json));
                    ctx.spawn(
                        async move {
                            addr.send(ResetCrashGame).await.unwrap();
                        }
                        .into_actor(act),
                    );
                }
            }
        });
    }
    fn reset_game(&mut self) {
        self.crashed = false;
        self.crash_point = None;
        self.started_at = None;
        self.round_id = None;
        self.public_seed = None;
        self.private_seed = None;
        self.interval_active = false;
        self.players.clear();
    }

    fn gen_crash_point(&mut self, public_seed: &str, private_seed: &str) -> f64 {
        let e = u32::MAX;
        let mut rng = OsRng;
        let h: u32 = rng.gen();

        if h % 33 == 0 {
            return 1.0;
        }

        let combined_seed = format!("{}{}", public_seed, private_seed);

        let mut hasher = Sha256::new();
        hasher.update(combined_seed);
        let result = hasher.finalize();

        let result_num = u32::from_be_bytes([result[0], result[1], result[2], result[3]]);
        1.00 + (((40.0 * (e as f64 - result_num as f64)) / (result_num as f64)).floor() / 100.0)
            as f64
    }

    fn update_game(&mut self, _ctx: &mut Context<Self>) {
        if let (Some(time), Some(crash_point)) = (self.started_at, self.crash_point) {
            let elapsed: f64 = time.elapsed().as_secs_f64();
            let mut growth_factor = 0.06;
            if self.multiplier >= 2.00 {
                growth_factor = 0.12;
            } else if self.multiplier >= 5.00 {
                growth_factor = 0.20;
            } else if self.multiplier >= 10.00 {
                growth_factor = 0.26;
            } else if self.multiplier >= 20.00 {
                growth_factor = 0.3;
            }
            let potential_multiplier = round_to(1.00 + ((elapsed * growth_factor).powf(1.9)), 2);
            if potential_multiplier >= crash_point {
                self.crashed = true;
                self.multiplier = crash_point;
            } else {
                self.multiplier = potential_multiplier;
            }
            println!(
                "multiplier: {}, crash: {:?}",
                self.multiplier, self.crash_point
            );
            if self.crashed {
                self.interval_active = false;
                self.crashed = true;
            }
        }
    }
}

fn round_to(value: f64, decimal_places: u32) -> f64 {
    let factor = 10f64.powi(decimal_places as i32);
    (value * factor).round() / factor
}
