use std::sync::{Arc, Mutex};

use rmt::{self, Origin, http::{Gate, instance::Encryption}, http_request, http_bind_worker, warn, rmtm};
mod definitions;
use definitions::*;

#[derive(Clone)]
pub struct ServiceWorker { 
    last_message: Arc<Mutex<String>>,
    http_client: reqwest::Client
}

#[rmtm::http_gate( MyService::Msg | ServiceWorker )]
async fn process(self, worker: &Self::W) -> Result<Self::Response, rmt::Error> {
    let msg: String = self.msg.chars().rev().collect();
    let mut last_msg_guard = worker.last_message.lock().unwrap();
    let last_msg = last_msg_guard.clone();
    *last_msg_guard = msg.clone();
    
    Ok(Self::Response {
        msg,
        last_msg
    })
}

#[rmtm::http_gate( MyService::Ping | ServiceWorker )]
async fn process(self, _worker: &Self::W) -> Result<Self::Response, rmt::Error> {
    Ok(Self::Response { })
}

#[rmtm::http_gate( MyService::Hello | ServiceWorker )]
async fn process(self, worker: &Self::W) -> Result<Self::Response, rmt::Error> {
    let new_msg = http_request! { 
        SERVICE_CONTEXT | (worker.http_client.clone()) 
        MyService : Msg { msg: self.msg }
    }
        .await
        .map(|res| res.msg + "1")
        .map_err(|e| warn!("{e}"))
        .unwrap_or("NO MSG!".to_string());
    
    Ok(Self::Response { 
        msg: new_msg
    })
}

impl rmt::http::Worker for ServiceWorker {
    http_bind_worker!{ SERVICE_CONTEXT | MyService }

    async fn middleware_pre(&self, request: actix_web::dev::ServiceRequest) 
            -> Result<actix_web::dev::ServiceRequest, rmt::Error> 
    {       
        println!("New request! {}", request.peer_addr().unwrap());

        Ok(request)
    }
}


#[rmtm::main(protocol = "http")]
async fn main() {
    rmt::logger::set_log_level(rmt::logger::LogLevel::Info);
    let service_worker = ServiceWorker { 
        last_message: Arc::new(Mutex::new(String::new())),
        http_client: reqwest::Client::new()
    };

    rmt::http::Instance::new(service_worker)
        .set_encryption(Encryption::None)
        .set_workers_count(2)
        // Will block local requests!
        // .set_allowed_origins(vec![Origin::Remote { ip: "122.12.52.12", port: 0 }]);
        // Only local requests!
        .set_allowed_origins(vec![Origin::Local { port: 0 }])
        .run()
        .await
        .expect("Error in main func");
}
