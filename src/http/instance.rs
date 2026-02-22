use std::sync::Arc;
use actix_web::{App, HttpServer, body::BoxBody, dev::{self, ServiceResponse}, middleware::{self, from_fn}, web};
use log::{error, info, trace, warn};

use crate::{Origin, gates::{GateErrorResponse, GateResult}, http::Worker, logger::init_logger, origin::AllowedOrigins};

pub enum Encryption {
    TLS(rustls::ServerConfig),
    SSL(openssl::ssl::SslAcceptorBuilder),
    None
}

/** **Service Instance**

    creates an instance to run the worker.
 */
pub struct Instance<W: Worker> {
    worker: W,
    internal: bool,
    allowed_origins: Option<Vec<Origin>>,
    encryption: Encryption,
    workers_count: usize   // 0 - automatic by actix
}

async fn processor<W: Worker>(data: web::Json<W::GReq>, worker: web::Data<W>) -> web::Json<GateResult<W::GRes>>  {
    let data = data.into_inner();

    let response = worker 
        .process_request(data)
        .await;

    if let Err(err) = &response {
        error!("Error occured in instance processor. {}", err.to_string())
    }

    web::Json(
        response
            .map(|g| GateResult::Ok(g))
            .unwrap_or_else(|err| GateResult::Err(GateErrorResponse::from(err)))
    )
}

async fn middleware<W>(
    request: dev::ServiceRequest, 
    next: middleware::Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, actix_web::Error>
    where W: Worker + 'static
{
    let origin = request.connection_info()
        .peer_addr()
        .ok_or_else( || {
            warn!("No peer address in middleware");
            actix_web::error::ErrorInternalServerError("No peer adress")
        })?
        .to_string();

    info!("{}", origin);

    trace!("New request from origin {}.", origin);

    let allowed_origins = request.app_data::<web::Data<Option<Arc<AllowedOrigins>>>>()
        .ok_or_else(|| {
            warn!("No allowed origing in middleware");
            actix_web::error::ErrorInternalServerError("Data error")
        })?;


    if let Some(origins) = allowed_origins.as_deref() { 
        if !origins.contains(&origin) {
            info!("New request from unlisted origin {}!", origin);
            return Ok(request.into_response(dev::Response::bad_request()));
        }
    };


    let worker = request.app_data::<web::Data<W>>()
        .ok_or_else(|| {
            warn!("No worker in middleware");
            actix_web::error::ErrorInternalServerError("Data error")
        })?
        .clone();

    let request = 
    if let Ok(request) = worker.get_ref()
        .middleware_pre(request)
        .await
        .map_err(|err| {
            warn!("Worker pre middleware errored! {}", err.to_string());
        }) { request } 
    else {
        return Err(actix_web::error::ErrorInternalServerError("Worker pre middleware"))
    };

 
    let response = next.call(request).await?;

    let response = 
    if let Ok(response) = worker.get_ref()
        .middleware_post(response)
        .await
        .map_err(|err| {
            warn!("Worker post middleware errored! {}", err.to_string());
        }) { response } 
    else {
        return Err(actix_web::error::ErrorInternalServerError("Worker post middleware"))
    };

    Ok(response)
}


impl<W> Instance<W> 
    where W: Worker + 'static
{
    pub fn new(worker: W) -> Self {
        init_logger();
        let internal = worker.context_ref().internal;

        Self {
            internal,
            worker,
            allowed_origins: None,
            encryption: Encryption::None,
            workers_count: 0
        }
    }

    pub fn set_encryption(&mut self, enc: Encryption) {
        self.encryption = enc;
    }

    pub fn set_workers_count(&mut self, count: usize) {
        self.workers_count = count;
    }

    /** Makes instance internal. Only requests from allowed origins are accepted.
        If allowed origins is ```None```, request from any origin will be accepted
    */
    pub fn set_allowed_origins(&mut self, allowed_origins: Vec<Origin>) {
        self.allowed_origins = Some(allowed_origins);
    }

    pub async fn run(self) -> std::io::Result<()> {
        let origin = self.worker.context_ref().origin();
        let ip = origin.ip();
        let port = origin.port();

        let worker_state = web::Data::new(self.worker.clone());

        let route_path = match self.internal {
            true => "/internal-request",
            false => "/request"
        };

        let allowed_origins = web::Data::new(
            self.allowed_origins.as_ref()
            .map(|o| Arc::new(AllowedOrigins::from(o)))
        );

        info!("Starting http server on {}:{}", ip, port);

        let server = HttpServer::new(move || {
            App::new()
                .app_data(worker_state.clone())
                .app_data(allowed_origins.clone())
                .wrap(from_fn(middleware::<W>))
                .route(route_path, web::post().to(processor::<W>))
        });

        let server = if self.workers_count != 0 {
            server.workers(self.workers_count)
        } else {
            server
        };

        match self.encryption {
            Encryption::None => server.bind((ip, port)),
            Encryption::SSL(ssl) => server.bind_openssl((ip, port), ssl),
            Encryption::TLS(tls) => server.bind_rustls_0_23((ip, port), tls)
        }?.run().await
    }
}