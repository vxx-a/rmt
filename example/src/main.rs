use std::sync::{Arc, Mutex};

use chrono::Local;
use rmt::{self, Origin, http::instance::Encryption};
mod definitions;
use definitions::*;

#[derive(Clone)]
struct ServiceWorker { 
    pub last_message: Arc<Mutex<String>>
}

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

                *self.last_message.lock().unwrap() = msg.clone();

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
            },
            Self::GReq::Last { } => {
                let msg = self.last_message.lock().unwrap().to_string();

                Ok(Self::GRes::Last { msg })
            }
        }
    }

    async fn middleware_pre(&self, request: actix_web::dev::ServiceRequest) 
            -> Result<actix_web::dev::ServiceRequest, rmt::Error> 
    {       
        println!("New request! {}", request.peer_addr().unwrap());

        Ok(request)
    }
}


#[rmt::rmtm::main(protocol = "http")]
async fn main() {
    rmt::logger::set_log_level(rmt::logger::LogLevel::Info);
    let service_worker = ServiceWorker { 
        last_message: Arc::new(Mutex::new(String::new()))
    };

    let mut instance = rmt::http::Instance::new(service_worker);

    instance.set_encryption(Encryption::None);
    instance.set_workers_count(2);
    
    // Will block local requests!
    // instance.set_allowed_origins(vec![Origin::Remote { ip: "122.12.52.12", port: 0 }]);
    
    // Only local requests!
    instance.set_allowed_origins(vec![Origin::Local { port: 0 }]);

    instance.run()
        .await
        .expect("Error in main func");
}
