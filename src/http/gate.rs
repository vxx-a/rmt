use crate::{Error, Payload, error::ServiceError, http::Worker};

// Gate which has request, response and a processor
pub trait Gate: Payload {
    type Response: Payload;
    type W: Worker;

    #[allow(unused_variables, async_fn_in_trait)]
    async fn process(self, worker: &Self::W) -> Result<Self::Response, Error>
    {
        Err(Error::Service(ServiceError::NotImplemented))
    }
}

// Marker for requests enum
pub trait RequestGatesMarker: Payload { }
// Marker for responses enum
pub trait ResponseGatesMarker: Payload { }

// Combined trait of requests and responses
pub trait Service {
    type Requests: RequestGatesMarker;
    type Responses: ResponseGatesMarker;
}