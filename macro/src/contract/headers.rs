use crate::meta::{Kind, PatMetas};
use crate::util::http_mod_path;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::LitStr;

pub struct HeaderTokens<'a> {
    metas: &'a PatMetas,
    headers: Vec<(LitStr, LitStr)>,
}

impl<'a> HeaderTokens<'a> {
    pub fn new(metas: &'a PatMetas, headers: Vec<(LitStr, LitStr)>) -> Self {
        Self { metas, headers }
    }
}

impl ToTokens for HeaderTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // attribute headers
        let http_mod = http_mod_path();
        let stream = match self.headers.is_empty() {
            true => quote! {
                let __request = match self.default_headers() {
                    Some(headers) => __request.headers(headers.clone()),
                    None => __request
                };
            },
            false => {
                let headers = self.headers.iter()
                    .map(|(name, value)| quote! { (#http_mod::HeaderName::from_static(#name), #http_mod::HeaderValue::from_static(#value)) });
                quote! {
                    let __request = {
                        let mut headers = #http_mod::HeaderMap::from_iter([#(#headers),*]);
                        if let Some(default_headers) = self.default_headers() {
                            default_headers.iter().for_each(|(k, v)| {
                                headers.insert(k.clone(), v.clone());
                            })
                        }
                        __request.headers(headers)
                    };
                }
            }
        };
        tokens.extend(stream);

        // argument headers
        let stream = self
            .metas
            .iter()
            .filter(|meta| meta.kind == Kind::Header)
            .map(|meta| {
                let header_name = match &meta.rename {
                    None => meta.ident.to_string(),
                    Some(rename) =>rename.value()
                };
                let header_value = &meta.ident;
                quote! { let __request = __request.header(stringify!(#header_name), #header_value); }
            });
        tokens.extend(stream);
    }
}
