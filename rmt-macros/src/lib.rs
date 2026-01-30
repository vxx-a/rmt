use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemEnum, ItemFn, parse_macro_input};

mod attribute;
use attribute::*;

#[proc_macro_attribute]
pub fn gates(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);

    let vis = &input.vis;
    let ident = &input.ident;
    let variants = &input.variants;
    let generics = &input.generics;

    quote! {
        #[derive(::serde::Serialize, ::serde::Deserialize, Clone)]
        #[serde(tag = "gate")]
        #vis enum #ident #generics {
            #variants
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as MainArgs);
    let input = parse_macro_input!(item as ItemFn);

    let block = input.block;
    let attrs = input.attrs;

    let expanded = match args.protocol {
        Protocol::Http => quote! {
            #(#attrs)*
            fn main() {
                actix_web::rt::System::new().block_on(async #block)
            }
        },

        Protocol::Websocket => quote! {
            #(#attrs)*
            fn main() {
                tokio::runtime::Runtime::new()
                    .expect("failed to create runtime")
                    .block_on(async #block)
            }
        },
    };

    expanded.into()
}