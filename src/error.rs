use std::fmt::Display;
use crate::http;

#[derive(Clone, Debug)]
pub enum Error {
    Http(http::error::Error),
    Websocket,
    Service(ServiceError),
    Custom(Option<String>)
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(err) => write!(f, "Http error. {:?}", err),
            Self::Websocket => write!(f, "Websocket error."),
            Self::Service(err) => write!(f, "Service error. {:?}", err),
            Self::Custom(err) => write!(f, "Custom Error. {}", err.clone().unwrap_or("empty".to_string()))
        }
    }
}

#[derive(Clone, Debug)]
pub enum ServiceError {
    ServiceRequestTimeout,
    JSONParseError(String),
    WrongGate
}