use actix::{clock::Instant, prelude::*};
use hex;
use rand::rngs::OsRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, time::Duration};
use uuid::Uuid;

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
        let addr = ctx.address();

        ctx.spawn(
            async move {
                let crash_game = CrashGame::new(addr).start();

                let res = crash_game.send(StartCrashGame).await;
                match res {
                    Ok(_) => println!("Started crash game"),
                    Err(e) => println!("Error starting crash game: {}", e),
                }
            }
            .into_actor(self),
        );
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
                    addr.do_send(ClientMessage::Text(txt.clone()));
                }
            }
            ClientMessage::Json(val) => {
                let json_string = serde_json::to_string(&val);
                match json_string {
                    Ok(json_str) => {
                        for addr in self.sessions.values() {
                            addr.do_send(ClientMessage::Text(json_str.clone()));
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

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartCrashGame;

impl Handler<StartCrashGame> for CrashGame {
    type Result = ();

    fn handle(&mut self, _msg: StartCrashGame, ctx: &mut Self::Context) -> Self::Result {
        self.game_started = true;
        self.run_game_loop(ctx);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ResetGame;

impl Handler<ResetGame> for CrashGame {
    type Result = ();

    fn handle(&mut self, _msg: ResetGame, ctx: &mut Self::Context) -> Self::Result {
        println!("Reseting game");
        self.reset_game();

        ctx.run_later(Duration::from_secs(3), |act, ctx| {
            act.run_game_loop(ctx);
        });
    }
}

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
    pub interval_active: bool,
    crash_point: Option<f64>
}

impl Actor for CrashGame {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
struct DelayedTaskCompleted;

impl Handler<DelayedTaskCompleted> for CrashGame {
    type Result = ();

    fn handle(&mut self, _msg: DelayedTaskCompleted, _ctx: &mut Self::Context) -> Self::Result {
        println!("{:?}", self);
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
            crash_server_addr,
            started_at: None,
            round_id: None,
            private_seed: None,
            public_seed: None,
            interval_active: false,
            crash_point: None
        }
    }
    fn run_game_loop(&mut self, ctx: &mut Context<Self>) {
        let addr = ctx.address();
    
        let round_id = Uuid::new_v4().to_string();
        let public_seed: [u8; 32] = OsRng.gen();
        let public_seed_hex = hex::encode(public_seed);
        let private_seed: [u8; 32] = OsRng.gen();
        let private_seed_hex = hex::encode(private_seed);
        self.round_id = Some(round_id.clone());
        self.public_seed = Some(public_seed_hex.clone());
        self.private_seed = Some(private_seed_hex.clone());
        if let (Some(private_seed), Some(public_seed)) = (&self.private_seed, &self.public_seed) {
            let crash_point = get_crash_point(private_seed, public_seed);
            self.crash_point = Some(crash_point);
            println!("Crash point: {}", crash_point);
            self.started_at = Some(Instant::now());
            self.interval_active = true;
    
            ctx.run_interval(Duration::from_millis(16), move |act, ctx| { // run more frequently to check the precise time
                if act.interval_active {
                    act.update_game(ctx);
    
                    if act.crashed {
                        act.interval_active = false;
                        let addr = ctx.address();
                        ctx.spawn(
                            async move {
                                addr.send(ResetGame).await.unwrap();
                            }
                            .into_actor(act),
                        );
                    }
                }
            });
        }
    }
    fn reset_game(&mut self) {
        self.players.clear();
        self.multiplier = 1.0;
        self.crashed = false;
        self.is_timer_started = false;
        self.game_started = false;
        self.started_at = None;
        self.round_id = None;
        self.private_seed = None;
        self.public_seed = None;
        self.interval_active = false;
        self.crash_point = None; 
    }
    fn update_game(&mut self, ctx: &mut Context<Self>) {
        if let (Some(time), Some(crash_point)) = (self.started_at, self.crash_point) {
            let elapsed = time.elapsed().as_secs_f64();
            let growth_factor = 0.26;
    
            let potential_multiplier = 1.0 + (elapsed * growth_factor).powf(1.5);
            
            if potential_multiplier > crash_point {
                self.multiplier = crash_point;
            } else {
                self.multiplier = potential_multiplier;
            }
    
            println!("time: {:.7}, multiplier: {:.2}, crash on: {}", elapsed,self.multiplier,crash_point);
    
            self.crash_server_addr
                .do_send(ClientMessage::Json(ClientMessageJson {
                    message: format!("Multiplier: {:.2}", self.multiplier),
                    message_type: "multiplier".to_string(),
                }))
                ;
            if self.multiplier >= crash_point {
                println!("Crash point: {:.2}", crash_point);
                self.crashed = true;
            }
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
