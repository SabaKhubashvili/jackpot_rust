use std::time::Duration;

use actix::ActorContext;
use actix::AsyncContext;
use actix::{clock::Instant, Actor};
use actix_web_actors::ws;

pub struct CoinflipWs {
    pub session_id: i32,
    // pub addr: Addr<CoinflipServer>,
    pub hb: Instant,
    pub user_id: i32,
    pub name: Option<String>,
}

impl Actor for CoinflipWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
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
