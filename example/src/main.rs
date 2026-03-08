use std::sync::{Arc, Mutex};

use log::warn;
use rmt::{self, Origin, http::{Gate, instance::Encryption}, http_bind_ctx, http_bind_service, http_request, rmtm};
mod definitions;
use definitions::*;

#[derive(Clone)]
struct ServiceWorker { 
    last_message: Arc<Mutex<String>>,
    http_client: reqwest::Client
}

#[rmtm::http_gate( MyService : Msg | ServiceWorker )]
async fn process(request: Self::Request, worker: &Self::W) -> Result<Self::Response, rmt::Error> {
    let msg: String = request.msg.chars().rev().collect();
    let last_msg = worker.last_message.lock().unwrap().to_string();
    *worker.last_message.lock().unwrap() = msg.clone();
    
    Ok(Self::Response {
        msg,
        last_msg
    })
}

#[rmtm::http_gate( MyService : Ping | ServiceWorker )]
async fn process(_request: Self::Request, _worker: &Self::W) -> Result<Self::Response, rmt::Error> {
    Ok(Self::Response { })
}

#[rmtm::http_gate( MyService : Hello | ServiceWorker )]
async fn process(request: Self::Request, worker: &Self::W) -> Result<Self::Response, rmt::Error> {
    let new_msg = http_request! { 
        SERVICE_CONTEXT | (worker.http_client.clone()) MyService : Msg { msg: request.msg }
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
    http_bind_service!{ MyService }
    http_bind_ctx!{ SERVICE_CONTEXT }

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
