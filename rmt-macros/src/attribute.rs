use syn::{parse::Parse, parse::ParseStream, LitStr, Token};

pub enum Protocol {
    Http,
    Websocket,
}

pub struct MainArgs {
    pub protocol: Protocol
}

impl Parse for MainArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;

        if ident != "protocol" {
            return Err(input.error("expected `protocol`"));
        }

        input.parse::<Token![=]>()?;
        let value: LitStr = input.parse()?;

        let protocol = match value.value().as_str() {
            "http" => Protocol::Http,
            "websocket" => Protocol::Websocket,
            _ => {
                return Err(syn::Error::new(
                    value.span(),
                    "protocol must be `http` or `websocket`",
                ))
            }
        };

        Ok(MainArgs { protocol })
    }
}

pub struct HTTPGateArgs {
    pub gate: syn::Ident,
    pub service: syn::Ident,
    pub worker: syn::Type
}

impl Parse for HTTPGateArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let service: syn::Ident = input.parse()?;

        input.parse::<Token![:]>()?;
        let gate: syn::Ident = input.parse()?;

        input.parse::<Token![|]>()?;
        let worker: syn::Type = input.parse()?;

        Ok(Self {
            gate: gate,
            service: service,
            worker: worker
        })
    }
}