use std::{collections::HashSet, sync::Arc};

const LOCALHOST: &str = "127.0.0.1";


pub enum Origin {
    Local { port: u16 },
    IP { addr: &'static str, port: u16 },
    Host { host: &'static str, port: u16 }
}

impl Origin {
    pub fn host(&self) -> &'static str {
        match self {
            Origin::Local { port: _ } => LOCALHOST,
            Origin::IP { addr: ip, port: _ } => ip,
            Origin::Host { host , port: _ } => host
        }
    }

    pub(crate) fn self_host(&self) -> &'static str {
        LOCALHOST
    }

    pub fn port(&self) -> u16 {
        match self {
            Origin::Local { port } => *port,
            Origin::IP { addr: _, port } => *port,
            Origin::Host { host: _, port } => *port
        }
    }
}

#[derive(Clone)]
pub struct AllowedOrigins {
    origins: Arc<HashSet<String>>
}

impl AllowedOrigins {
    pub fn contains(&self, origin: &str) -> bool {
        self.origins.contains(origin)
    }
}

impl From<&Vec<Origin>> for AllowedOrigins {
    fn from(value: &Vec<Origin>) -> Self {
        let mut set = HashSet::new();

        value.into_iter()
            .map(|origin| origin.host())
            .for_each(|ip| {
                set.insert(ip.to_string());
            });

        Self {
            origins: Arc::new(set)
        }
    }
}