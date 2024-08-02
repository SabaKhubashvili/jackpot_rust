use actix::{clock::Instant, prelude::*};
use rand::Rng;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::{collections::HashMap, time::Duration};
use uuid::Uuid;
use hex;

#[derive(Debug)]
pub struct CrashGame {
    pub multiplier: f64,
    pub is_timer_started: bool,
    pub game_started: bool,
    pub crashed: bool,
    pub players: Vec<Bet>,
    pub started_at: Option<Instant>,
    pub crash_server_addr: Addr<CrashServer>,
    pub round_id: Option<String>,
    pub private_seed: Option<String>,
    pub public_seed: Option<String>,
}

impl Actor for CrashGame {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartGameLoop;

impl Handler<StartGameLoop> for CrashGame {
    type Result = ();

    fn handle(&mut self, _msg: StartGameLoop, ctx: &mut Self::Context) -> Self::Result {
        println!("Starting game loop");
        self.run_game_loop(ctx);
    }
}

impl CrashGame {
    pub fn new(crash_server_addr: Addr<CrashServer>) -> Self {
        CrashGame {
            multiplier: 1.0,
            is_timer_started: false,
            game_started: false,
            crashed: false,
            players: Vec::new(),
            started_at: None,
            crash_server_addr,
            round_id: None,
            private_seed: None,
            public_seed: None,
        }
    }

    fn run_game_loop(&mut self, ctx: &mut Context<Self>) {
        // Generate seeds and round ID
        let round_id = Uuid::new_v4().to_string();
        let public_seed: [u8; 32] = OsRng.gen();
        let public_seed_hex = hex::encode(public_seed);
        let private_seed: [u8; 32] = OsRng.gen();
        let private_seed_hex = hex::encode(private_seed);

        self.round_id = Some(round_id.clone());
        self.public_seed = Some(public_seed_hex.clone());
        self.private_seed = Some(private_seed_hex.clone());

        // Send public seed and round ID to players
        self.crash_server_addr.do_send(ClientMessage::Json(ClientMessageJson {
            message: format!("Round ID: {}\nPublic Seed: {}", round_id, public_seed_hex),
            message_type: "start".to_string(),
        }));

        // Start game logic here
        self.started_at = Some(Instant::now());
        self.game_started = true;
        self.is_timer_started = true;

        if let (Some(public_seed), Some(private_seed)) = (&self.public_seed, &self.private_seed) {
            let crash_point = get_crash_point(public_seed, private_seed);
            ctx.run_interval(Duration::from_millis(100), move |act, ctx| {
                act.update_game(ctx);

                if act.multiplier >= crash_point {
                    act.crashed = true;
                }

                if act.crashed {
                    act.crash_server_addr.do_send(ClientMessage::Json(ClientMessageJson {
                        message: format!("Crashed at Multiplier: {:.2}", act.multiplier),
                        message_type: "crash".to_string(),
                    }));

                    // Reveal private seed after crash
                    act.crash_server_addr.do_send(ClientMessage::Json(ClientMessageJson {
                        message: format!("Private Seed: {}", act.private_seed.clone().unwrap()),
                        message_type: "seed_reveal".to_string(),
                    }));

                    ctx.stop();  // Stop the interval loop after crashing
                }
            });
        }
    }

    fn reset_game(&mut self, ctx: &mut Context<Self>) {
        println!("Resetting game, waiting 15 sec");
        ctx.run_later(Duration::from_secs(1), move |act, run_ctx| {
            act.players.clear();
            act.multiplier = 1.0;
            act.crashed = false;
            act.is_timer_started = false;
            act.game_started = false;
            act.started_at = None;
            act.round_id = None;
            act.private_seed = None;
            act.public_seed = None;

            println!("Starting game again after reset");
            act.crash_server_addr.do_send(StartGameLoop);  // Send the message to start a new game loop
        });
    }

    fn update_game(&mut self, _ctx: &mut Context<Self>) {
        if let Some(time) = self.started_at {
            let elapsed = time.elapsed().as_secs_f64();
            self.multiplier = 1.0 + elapsed.powf(1.3);

            self.crash_server_addr
                .do_send(ClientMessage::Json(ClientMessageJson {
                    message: format!("Multiplier: {:.2}", self.multiplier),
                    message_type: "multiplier".to_string(),
                }));

            println!("Multiplier: {}", self.multiplier);
        }
    }
}

// Function to get crash point
fn get_crash_point(public_seed: &str, private_seed: &str) -> f64 {
    let e = u32::MAX;
    let mut rng = OsRng;
    let h: u32 = rng.gen();

    // 2% chance of an instant crash
    if h % 33 == 0 {
        return 1.0;
    }

    // Calculating the crash multiplier using the seeds
    let combined_seed = format!("{}{}", public_seed, private_seed);
    let mut hasher = Sha256::new();
    hasher.update(combined_seed);
    let result = hasher.finalize();
    let result_num = u32::from_be_bytes([result[0], result[1], result[2], result[3]]);

    1.00 + (((40.0 * (e as f64 - result_num as f64)) / (result_num as f64)).floor() / 100.0) as f64
}

// Define other necessary structs and enums here
#[derive(Debug)]
pub struct Bet {
    pub user_id: i32,
    pub bet_amount: f64,
    pub cashed_out: bool,
}

pub struct CrashServer {
    pub sessions: HashMap<i32, Recipient<ClientMessage>>,
    pub crash_game: Option<Addr<CrashGame>>,
}

impl CrashServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            crash_game: None,
        }
    }
}

impl Actor for CrashServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Starting crash server");
        let crash_game = CrashGame::new(ctx.address());
        let crash_addr = crash_game.start();
        self.crash_game = Some(crash_addr);
        self.crash_game.as_ref().unwrap().do_send(StartGameLoop);
    }
}

impl Handler<StartGameLoop> for CrashServer {
    type Result = ();

    fn handle(&mut self, _msg: StartGameLoop, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(crash_game) = &self.crash_game {
            crash_game.do_send(StartGameLoop);
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub user_id: i32,
    pub addr: Recipient<ClientMessage>,
}

impl Handler<Connect> for CrashServer {
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

impl Handler<Disconnect> for CrashServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.remove(&msg.user_id);
    }
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub enum ClientMessage {
    Text(String),
    Json(ClientMessageJson),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientMessageJson {
    pub message: String,
    pub message_type: String,
}

impl Handler<ClientMessage> for CrashServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg.clone() {
            ClientMessage::Text(txt) => {
                for addr in self.sessions.values() {
                    addr.do_send(ClientMessage::Text(txt.clone()))
                }
            }
            ClientMessage::Json(val) => {
                let json_string = serde_json::to_string(&val);
                match json_string {
                    Ok(json_str) => {
                        for addr in self.sessions.values() {
                            addr.do_send(ClientMessage::Text(json_str.clone()))
                        }
                    }
                    Err(e) => {
                        eprintln!("Error converting JSON: {}", e);
                    }
                }
            }
        }
    }
}
