use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::Error;

#[derive(Serialize, Deserialize, Clone)]
pub struct GateErrorResponse {
    error: String
}

/** **Service Gates**

    Compound trait for ```serde::Serialize```, ```serde::Deserialize``` and ```Clone```
 */
pub trait Payload: Serialize + DeserializeOwned + Clone { }

impl From<Error> for GateErrorResponse {
    fn from(value: Error) -> Self {
        Self { error: value.to_string() }
    }
}

impl Payload for GateErrorResponse { }

pub enum GateResult<G: Payload> {
    Ok(G),
    Err(GateErrorResponse)
}

impl<G: Payload> Serialize for GateResult<G> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer 
    {
        match self {
            Self::Ok(g) => g.serialize(serializer),
            Self::Err(err) => err.serialize(serializer)
        }
    }
}