#[rmt::rmtm::gates]
pub enum ExGatesReq {
    Ping { },
    Hello { msg: String },
    HelloToMe { msg: String },
    Time { },
    Last { }
}

#[rmt::rmtm::gates]
pub enum ExGatesRes {
    Pong { },
    Hello { msg: String },
    Time { time: String },
    Last { msg: String }
}

pub const SERVICE_CONTEXT: rmt::http::Context<ExGatesReq, ExGatesRes> = 
    rmt::http::Context::new(rmt::Origin::Local { port: 2020 }, false);