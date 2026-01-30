use crate::{Error, Origin, http::{Context, Gates}};

#[allow(async_fn_in_trait)]

/** **Service Worker** 
 
    defines interface for user to implement processing. Independent from Service Context.
    
    *Is used internally by Service Instance*
 */
pub trait Worker: Clone + Sync + Send {
    type GReq: Gates;
    type GRes: Gates;

    async fn process_request(&self, gates: Self::GReq) -> Result<Self::GRes, Error>;

    /** Should return reference on static context with same gates */
    fn context_ref(&self) -> &'static Context<Self::GReq, Self::GRes>;

    async fn middleware_pre(&self, request: actix_web::dev::ServiceRequest) 
        -> (actix_web::dev::ServiceRequest, Option<Error>) 
    {
        (request, None)
    }

    async fn middleware_post(&self, response: actix_web::dev::ServiceResponse) 
        -> (actix_web::dev::ServiceResponse, Option<Error>)
    {
        (response, None)
    }
}