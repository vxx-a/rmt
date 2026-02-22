# Rust Microservice Template
**Define Microservices as Static Types**

Built on top of actix.
Allows for http/https and websocket service protocols.
##### Philosophy
Describe your microservices once and use anywhere.

Make requests to other microservices by simply importing them.
### Definitions
`[Enum]`**Origin**  - service origin. Is relativistic to other services and depends on final architecture (local for small projects in one docker image, remote for big projects involving many docker images)

`[Trait]`**Gates** - api itself. For http request and response gates. For websockets incoming and outgoing.

`[Static Object]`**Context** - service description.

`[Trait]`**Worker** - trait for an user-defined worker object. Can store shareable data, should have a Context attached. User implements request processing in the trait implementation.

`[Object]`**Instance** - service instance, which operates on a worker object.

### Example
The [example crate](./example).

###### Service Definitions
```rust
// Request Gates
#[rmt::rmtm::gates]
pub enum GatesReq {
    Ping { },
    Hello { msg: String }
}

// Response Gates
#[rmt::rmtm::gates]
pub enum GatesRes {
    Pong { },
    Hello { msg: String }
}

// Describe a service with gates from above
// Assign localhost on port 2020
// Set false to internal flag (can recieve any request)
pub const SERVICE_CONTEXT: rmt::http::Context<GatesReq, GatesRes> = 
    rmt::http::Context::new(rmt::Origin::Local { port: 2020 }, false);
```
*This part can be accessed by other services.*

User is responsible for assigning some response to each request. Request Ping should return a response Pong, but it may not. It is user's responsibility to match the service response correctly. Stronger typization may be added in the future.

###### Service implementation
```rust
#[derive(Clone)]
struct ServiceWorker { 
	// Shareable data for example
    pub last_message: Arc<Mutex<String>>
}

impl rmt::http::Worker for ServiceWorker {
	// Assigning gates to your worker
	type GReq = GatesReq;
    type GRes = GatesRes;
    
    // User should return a reference to the 
    // static context defined before
    fn context_ref(&self) -> 
    &'static rmt::http::Context<Self::GReq, Self::GRes>;
    
    // The request processor.
    async fn process_request(&self, gates: Self::GReq) ->
    Result<Self::GRes, rmt::Error>;
    
    // Optional middleware implementation
    // Allows to manipulate the actix_web request
    async fn middleware_pre(&self, request: ServiceRequest) -> 
        Result<ServiceRequest, rmt::Error>;
        
    // Optional middleware implementation
    // Allows to manipulate the actix_web response
    async fn middleware_post(&self, response: ServiceResponse) -> 
        Result<ServiceResponse, rmt::Error>;
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
    let mut instance = rmt::http::Instance::new(service_worker);
    instance.set_workers_count(2);

    instance.run()
        .await
        .expect("Error in main func");
}
```

###### Requests to other services
```rust
use some_service::defs;

...
	defs::SERVICE_CONTEXT.request(http_client, defs::Request::Ping { })
		.await
...
```
