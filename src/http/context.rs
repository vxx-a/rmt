use std::{marker::PhantomData, time::Duration};
use crate::{Error, Origin, error::ServiceError, http::{self, Gates}};

const SERVICE_REQUEST_TIMEOUT: u64 = 5000;  // 5 seconds

/** **Service Context**

    describes a microservice origin and its gates. 
 */
pub struct Context<GReq: Gates, GRes: Gates> {
    pub(crate) origin: Origin,
    pub(crate) phantom: PhantomData<(GReq, GRes)>,
    pub(crate) internal: bool
}

impl<GRes: Gates, GReq: Gates> Context<GReq, GRes> {
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
    pub async fn request(&self, http_client: reqwest::Client, gate: GReq) -> Result<GRes, Error> {
        let path = if self.internal { "internal-request" } else { "request" };
        
        let raw = http_client.post(format!("http://{}:{}/{path}", self.origin.ip(), self.origin.port()))
            .json(&gate)
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