use crate::util::reqwest_mod_path;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

pub struct RequestTokens {
    method: Ident,
}

impl RequestTokens {
    pub fn new(method: Ident) -> Self {
        Self { method }
    }
}

impl ToTokens for RequestTokens {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let reqwest_mod = reqwest_mod_path();
        let method = &self.method;
        let stream = quote! {
            let __request = self.client().request(#reqwest_mod::Method::#method, __url);
        };
        tokens.extend(stream);
    }
}
