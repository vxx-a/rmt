use crate::{Error, http::{Context, gate::Service}};

#[allow(async_fn_in_trait)]

/** **Service Worker** 
 
    Defines interface for user to implement processing. Independent from Service Context.
    Worker can store data which can be accessed in every request. Because of this Clone + Sync + Send is required.
    
    Use binding macros inside for simplicity
    ```
    http_bind_worker!{ SERVICE_CONTEXT | MyService }
    ```

    Is called by Service Instance.
 */
pub trait Worker: Clone + Sync + Send {
    type S: Service;

    async fn matcher(&self, request: <Self::S as Service>::Requests)
        -> Result<<Self::S as Service>::Responses, Error>;

    fn context_ref(&self) -> &'static Context<Self::S>;

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