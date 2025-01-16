use crate::contract::path::PathParams;
use crate::meta::{Kind, PatMetas};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct UrlTokens<'a> {
    metas: &'a PatMetas,
    pattern: String,
}

impl<'a> UrlTokens<'a> {
    pub fn new(metas: &'a PatMetas, path: &'a str) -> syn::Result<Self> {
        let _ = PathParams::new(path).check(metas)?;
        let pattern = PathParams::get_format_pattern(path);
        Ok(Self { pattern, metas })
    }
}

impl ToTokens for UrlTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let metas = self
            .metas
            .iter()
            .filter(|meta| meta.kind == Kind::Path)
            .collect::<Vec<_>>();
        let pattern = &self.pattern;
        let stream = match metas.is_empty() {
            true => quote! {
                 let __url = match self.base_url() {
                    Some(base_url) => format!("{}{}", base_url, #pattern),
                    None => #pattern.to_string(),
                };
            },
            false => {
                let args = metas
                    .iter()
                    .map(|meta| &meta.ident)
                    .map(|arg| quote! { #arg });
                quote! {
                    let __url = {
                        let path = format!(#pattern, #(#args),*);
                        match self.base_url() {
                            Some(base_url) => format!("{}{}", base_url, path),
                            None => path,
                        }
                    };
                }
            }
        };
        tokens.extend(stream)
    }
}
