use crate::{Error, Payload, error::ServiceError, http::Worker};

// Gate which has request, response and a processor
pub trait Gate {
    type Request: Payload;
    type Response: Payload;
    type W: Worker;

    #[allow(unused_variables)]
    fn process(request: Self::Request, worker: &Self::W)
        -> impl Future<Output = Result<Self::Response, Error>> + Send
    {
        async { Err(Error::Service(ServiceError::NotImplemented)) }
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