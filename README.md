# Rust Microservice Template
##### Define Microservices as Static Types

Built on top of actix.
Allows for http/https and websocket service protocols.
##### Philosophy
Describe your microservices once and use anywhere.
Make requests to other microservices by simply importing them.
### Definitions
`[Enum]`**Origin**  - service origin. Is relativistic to other services and depends on final architecture (local for small projects in one docker image, remote for big projects involving many docker images).

`[Static Object]`**Context** - service description.

`[Trait]`**Gate** - Inputs and Outputs of a service.

`[Trait]`**Worker** - trait for an user-defined worker object. Can store shareable data, should have a Context attached.

`[Object]`**Instance** - service instance, which operates on a worker object.

### Example
The [example crate](./example).

###### HTTP Service Definitions
```rust
// Request / Response gates. MyService as a service name
http_gates!(MyService [
    Msg {
        request: { msg: String },
        response: { msg: String, last_msg: String }
    },
    Ping {
        request: { },
        response: { }
    },
    Hello {
        request: { msg: String },
        response: { msg: String }
    }
]);

// Local origin, non-internal
pub static SERVICE_CONTEXT: rmt::http::Context<MyService> = http_context![ ::2020 ];
```
*This part can be accessed by other services.*

User is assigning to each gate some request and response.

###### Gates implementation
```rust
#[rmtm::http_gate( MyService : Msg | ServiceWorker )]
async fn process(self, worker: &Self::W) -> Result<Self::Response, rmt::Error> {
    let msg: String = self.msg.chars().rev().collect();
    let last_msg = worker.last_message.lock().unwrap().to_string();
    *worker.last_message.lock().unwrap() = msg.clone();
    
    Ok(Self::Response {
        msg,
        last_msg
    })
}

#[rmtm::http_gate( MyService : Ping | ServiceWorker )]
async fn process(self, _worker: &Self::W) -> Result<Self::Response, rmt::Error> {
    Ok(Self::Response { })
}

#[rmtm::http_gate( MyService : Hello | ServiceWorker )]
async fn process(self, worker: &Self::W) -> Result<Self::Response, rmt::Error> {
    // Send requests to other services
    let new_msg = http_request! { 
        SERVICE_CONTEXT | (worker.http_client.clone()) 
        MyService : Msg { msg: request.msg }
    }
        .await
        .map(|res| res.msg + "1")
        .map_err(|e| warn!("{e}"))
        .unwrap_or("NO MSG!".to_string());
    
    Ok(Self::Response { 
        msg: new_msg
    })
}
```

###### Service implementation
```rust
#[derive(Clone)]
struct ServiceWorker { 
	// Shareable data for example
    pub last_message: Arc<Mutex<String>>
}

impl rmt::http::Worker for ServiceWorker {
    http_bind_worker! { SERVICE_CONTEXT | MyService }

    async fn middleware_pre(&self, request: actix_web::dev::ServiceRequest) 
            -> Result<actix_web::dev::ServiceRequest, rmt::Error> 
    {       
        println!("New request! {}", request.peer_addr().unwrap());

        Ok(request)
    }
}
```

###### Running your service
```rust
#[rmt::rmtm::main(protocol = "http")]
async fn main() {
	// Set the log level
    rmt::logger::set_log_level(rmt::logger::LogLevel::Info);
    
    // Create your worker
    let service_worker = ServiceWorker { 
        last_message: Arc::new(Mutex::new(String::new()))
    };

	// Create your instance from the worker
    rmt::http::Instance::new(service_worker)
        .set_workers_count(2);
        .run()
        .await
        .expect("Error in main func");
}
```

###### Requests to other services
```rust
use some_service::defs;

...
    http_request! { defs::SERVICE_CONTEXT | (http_client) 
    ServiceName : Method { fields } }
...
```
