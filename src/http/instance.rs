use std::sync::Arc;

use actix_web::{App, HttpServer, body::BoxBody, dev::{self, ServiceResponse}, middleware::{self, from_fn}, web};
use log::{error, info, warn};

use crate::{Origin, http::{Worker, gates::{GateErrorResponse, GateResult}}, logger::init_logger, origin::AllowedOrigins};

/** **Service Instance**

    creates an instance to run the worker.
 */
pub struct Instance<W: Worker> {
    worker: W,
    internal: bool,
    allowed_origins: Option<Vec<Origin>>
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
        .expect("Middleware expected peer address!")
        .to_string();

    info!("New request from origin {}.", origin);

    let allowed_origins = request.app_data::<web::Data<Option<Arc<AllowedOrigins>>>>()
        .expect("AllowedOrigins did not reach middleware!");


    if let Some(origins) = allowed_origins.as_deref() { 
        if !origins.contains(&origin) {
            warn!("New request from unlisted origin {}!", origin);
            return Ok(request.into_response(dev::Response::bad_request()));
        }
    };


    let worker = request.app_data::<web::Data<W>>()
        .expect("Worker did not react middleware")
        .clone();

    let (request, err) = worker.get_ref()
        .middleware_pre(request)
        .await;

    if let Some(err) = err {
        info!("Worker pre middleware errored! {}", err.to_string());
        return Ok(request.into_response(dev::Response::bad_request()));
    }
 
    let response = next.call(request).await?;

    let (response, err) = worker.get_ref()
        .middleware_post(response)
        .await;

    if let Some(err) = err {
        info!("Worker post middleware errored! {}", err.to_string());
        return Ok(response.into_response(actix_web::HttpResponse::Locked().finish()));
    }

    Ok(response)
}


impl<W> Instance<W> 
    where W: Worker + 'static
{
    pub fn new(worker: W) -> Self {
        init_logger();

        Self {
            internal: worker.context_ref().internal,
            worker,
            allowed_origins: None
        }
    }

    /** Makes instance internal. Only requests from allowed origins are accepted.
        If allowed origins is ```None```, request from any origin will be accepted
    */
    pub fn set_allowed_origins(&mut self, allowed_origins: Vec<Origin>) {
        self.allowed_origins = Some(allowed_origins);
    }

    pub async fn run(&self) -> std::io::Result<()> {
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

        HttpServer::new(move || {
            App::new()
                .app_data(worker_state.clone())
                .app_data(allowed_origins.clone())
                .wrap(from_fn(middleware::<W>))
                .route(route_path, web::post().to(processor::<W>))
        })
        .bind((ip, port))?
        .run()
        .await
    }
}