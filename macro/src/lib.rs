mod contract;
mod meta;
mod resource;
mod symbol;
mod util;

use crate::resource::Resource;
use contract::Contract;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::{parse_macro_input, ItemTrait, TraitItemFn};

#[proc_macro_attribute]
pub fn hadorn(args: TokenStream, stream: TokenStream) -> TokenStream {
    let resource = parse_macro_input!(args as Resource);
    let item = parse_macro_input!(stream as ItemTrait);
    resource
        .expand(item)
        .expect("expand attribute `resource` failed`")
        .into()
}

macro_rules! http_method {
    (get) => {
        "GET"
    };
    (post) => {
        "POST"
    };
    (put) => {
        "PUT"
    };
    (delete) => {
        "DELETE"
    };
    (head) => {
        "HEAD"
    };
    (options) => {
        "OPTIONS"
    };
    (trace) => {
        "TRACE"
    };
}

macro_rules! impl_methods {
    ($($method:ident),* $(,)?) => {
        $(
            #[doc = concat!("Auto generate http call for [`", http_method!($method), "`]")]
            #[proc_macro_attribute]
            pub fn $method(args: TokenStream, input: TokenStream) -> TokenStream {
                let item = parse_macro_input!(input as TraitItemFn);
                let mut contract = parse_macro_input!(args as Contract);
                contract.method = Some(Ident::new(http_method!($method), Span::call_site()));
                contract.expand(item).expect(concat!("expand attribute `", stringify!($method) ,"` failed")).into()
            }
        )*
    };
}

impl_methods!(get, post, put, delete, head, options, trace);
