use serde::{Deserialize, Serialize};


#[rmt::rmtm::gates]
pub enum ExGatesReq {
    Ping { },
    Hello { msg: String },
    HelloToMe { msg: String },
    Time { }
}

impl rmt::http::Gates for ExGatesReq { }

#[rmt::rmtm::gates]
pub enum ExGatesRes {
    Pong { },
    Hello { msg: String },
    Time { time: String }
}

impl rmt::http::Gates for ExGatesRes { }

pub const SERVICE_CONTEXT: rmt::http::Context<ExGatesReq, ExGatesRes> = 
    rmt::http::Context::new(rmt::Origin::Local { port: 2020 }, false);