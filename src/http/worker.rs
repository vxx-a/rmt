use crate::{Error, gates::Gates, http::Context};

#[allow(async_fn_in_trait)]

/** **Service Worker** 
 
    Defines interface for user to implement processing. Independent from Service Context.
    Worker can store data which can be accessed in every request. Because of this Clone + Sync + Send is required.
    
    Is called by Service Instance.
 */
pub trait Worker: Clone + Sync + Send {
    type GReq: Gates + 'static;
    type GRes: Gates + 'static;

    async fn process_request(&self, gates: Self::GReq) -> Result<Self::GRes, Error>;

    /** Should return reference on static context with same gates */
    fn context_ref(&self) -> &'static Context<Self::GReq, Self::GRes>;

    /** Function is ran before the request has been processed */
    async fn middleware_pre(&self, request: actix_web::dev::ServiceRequest) 
        -> Result<actix_web::dev::ServiceRequest, Error> 
    {
        Ok(request)
    }

    /** Function is ran after the request has been processed */
    async fn middleware_post(&self, response: actix_web::dev::ServiceResponse) 
        -> Result<actix_web::dev::ServiceResponse, Error>
    {
        Ok(response)
    }
}