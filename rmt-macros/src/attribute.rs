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