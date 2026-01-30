use chrono::{DateTime, Local};
use rmt;
mod definitions;
use definitions::*;

#[derive(Clone)]
struct ServiceWorker { }

impl rmt::http::Worker for ServiceWorker {
    type GReq = ExGatesReq;
    type GRes = ExGatesRes;

    fn context_ref(&self) -> &'static rmt::http::Context<Self::GReq, Self::GRes> {
        &SERVICE_CONTEXT
    }

    async fn process_request(&self, gates: Self::GReq) -> Result<Self::GRes, rmt::Error> {
        match gates {
            Self::GReq::Hello { msg } => {
                let msg = msg.chars()
                    .rev()
                    .fold(String::new(), |s, x| s + &x.to_string());

                Ok(Self::GRes::Hello { msg })
            },
            Self::GReq::Ping {  } => {
                Ok(Self::GRes::Pong {  })
            },
            Self::GReq::HelloToMe { msg } => {
                let http_client = reqwest::Client::new();

                let response = SERVICE_CONTEXT
                    .request(http_client, ExGatesReq::Hello { msg: msg.clone() })
                    .await;

                match response {
                    Ok(g) => Ok(g),
                    Err(e) => Ok(ExGatesRes::Hello { msg: e.to_string() })
                }
            },
            Self::GReq::Time { } => {
                let s = Local::now().to_string();

                Ok(Self::GRes::Time { time: s })
            }
        }
    }
}

const SERVICE_WORKER: ServiceWorker = ServiceWorker { };

#[rmt::rmtm::main(protocol = "http")]
async fn main() {
    rmt::logger::set_log_level(rmt::logger::LogLevel::Info);

    let instance = rmt::http::Instance::new(SERVICE_WORKER);

    instance.run()
        .await
        .expect("Error in main func");
}
