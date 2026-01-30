use std::{collections::HashSet, sync::Arc};


pub enum Origin {
    Local { port: u16 },
    Remote { ip: &'static str, port: u16 }
}

impl Origin {
    pub fn ip(&self) -> &'static str {
        if let Origin::Remote { ip , port: _ } = self {
            *ip
        } else {
            "127.0.0.1"
        }
    }

    pub fn port(&self) -> u16 {
        match self {
            Origin::Local { port } => *port,
            Origin::Remote { ip: _, port } => *port
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
            .map(|origin| origin.ip())
            .for_each(|ip| {
                set.insert(ip.to_string());
            });

        Self {
            origins: Arc::new(set)
        }
    }
}