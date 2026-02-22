use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::Error;

#[derive(Serialize, Deserialize, Clone)]
pub struct GateErrorResponse {
    error: String
}

/** **Service Gates**

    Compound trait for ```serde::Serialize```, ```serde::Deserialize``` and ```Clone```
 */
pub trait Gates: Serialize + DeserializeOwned + Clone + Sized { }

impl From<Error> for GateErrorResponse {
    fn from(value: Error) -> Self {
        Self { error: value.to_string() }
    }
}

impl Gates for GateErrorResponse { }

pub enum GateResult<G: Gates> {
    Ok(G),
    Err(GateErrorResponse)
}

impl<G: Gates> Serialize for GateResult<G> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer 
    {
        match self {
            Self::Ok(g) => g.serialize(serializer),
            Self::Err(err) => err.serialize(serializer)
        }
    }
}