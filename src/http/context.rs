use std::{marker::PhantomData, time::Duration};
use crate::{Error, Origin, error::ServiceError, http::{self, gate::{Gate, Service}}};

const SERVICE_REQUEST_TIMEOUT: u64 = 5000;  // 5 seconds

/** **Service Context**

    describes a microservice origin and binded service. 
 */
pub struct Context<S: Service> {
    pub(crate) origin: Origin,
    pub(crate) phantom: PhantomData<S>,
    pub(crate) internal: bool
}

impl<S: Service> Context<S> {
    pub const fn new(origin: Origin, internal: bool) -> Self {
        Self {
            origin,
            phantom: PhantomData,
            internal
        }
    }

    pub fn origin(&'static self) -> &'static Origin {
        &self.origin
    }

    /** Make request to a microservice by using context */
    pub async fn request<G>(&self, http_client: reqwest::Client, gate: G) 
        -> Result<G::Response, Error> 
    where 
        G: Gate + Into<<S as Service>::Requests>,
    {
        let path = if self.internal { "internal-request" } else { "request" };
        
        let raw = http_client.post(format!("http://{}:{}/{path}", self.origin.host(), self.origin.port()))
            .json(&Into::<S::Requests>::into(gate))
            .timeout(Duration::from_millis(SERVICE_REQUEST_TIMEOUT))
            .send()
            .await
            .map_err(|err| {
                if err.is_timeout() {
                    Error::Service(ServiceError::ServiceRequestTimeout)
                } else {
                    Error::Http(http::error::Error::Text(err.to_string()))
                }
            })?;

        raw.json().await
        .map_err(|err| Error::Service(ServiceError::JSONParseError(err.to_string())))
    }
}

#[macro_export]
macro_rules! http_request {
    {
        $context:ident | ($client:expr) $service_name:ident : 
        $gate_name:ident { $($req_field:ident : $req_v:expr ),* $(,)? }
    } => {
        $crate::paste::paste! {
            $context.request::<[<RMTHTTP $service_name $gate_name Req>]>(
                $client,
                [<RMTHTTP $service_name $gate_name Req>] { $( $req_field : $req_v ),* }
            )
        }
    };
}

/** Macro for http::Context creation

    Examples:
    ```
    [ ::2020 ]
    [ (i) ::2020 ]
    [ "126.92.24.2":2020 ]
    [ (i) "126.92.24.2":2020 ]
    [ H"my.website":2020 ]
    [ (i) H"my.website":2020 ]
    ```
    `(i)` marks internal
 */
#[macro_export]
macro_rules! http_context {
    [
        ::$port:expr
    ] => {
        $crate::http::Context::new($crate::Origin::Local { port: $port }, false)
    };
    [
        (i) ::$port:expr
    ] => {
        $crate::http::Context::new($crate::Origin::Local { port: $port }, true)
    };
    [
        $ip:literal:$port:expr
    ] => {
        $crate::http::Context::new($crate::Origin::IP { addr: $ip, port: $port }, false)
    };
    [
        (i) $ip:literal:$port:expr
    ] => {
        $crate::http::Context::new($crate::Origin::IP { addr: $ip, port: $port }, true)
    };
    [
        H$host:literal:$port:expr 
    ] => {
        $crate::http::Context::new($crate::Origin::Host { host: $host, port: $port }, false)
    };
    [
        (i) H$host:literal:$port:expr 
    ] => {
        $crate::http::Context::new($crate::Origin::Host { host: $host, port: $port }, true)
    }
}