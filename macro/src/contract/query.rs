use crate::meta::{Kind, PatMetas};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Lifetime, Type};

const LIFETIME: &str = "'a";

pub struct QueryTokens<'a>(&'a PatMetas);

impl<'a> QueryTokens<'a> {
    pub fn new(metas: &'a PatMetas) -> Self {
        Self(metas)
    }
}

impl ToTokens for QueryTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let metas = self
            .0
            .iter()
            .filter(|meta| meta.kind == Kind::Query)
            .collect::<Vec<_>>();

        // query struct
        let stream = match metas.is_empty() {
            false => {
                let lifetime = match metas.iter().any(|meta| meta.reference) {
                    true => Some(Lifetime::new(LIFETIME, Span::call_site())),
                    false => None,
                };
                let fields = metas.iter().map(|meta| {
                    let ident = &meta.ident;
                    let mut ty = meta.ty.clone();
                    if let Type::Reference(reference) = ty.as_mut() {
                        reference.lifetime = lifetime.clone();
                    }
                    let ty = match meta.optional {
                        true => quote! { Option<#ty> },
                        false => quote! { #ty },
                    };
                    let serde_rename = meta
                        .rename
                        .as_ref()
                        .map(|rename| quote! { #[serde(rename = #rename)] });
                    quote! {
                        #serde_rename
                        #ident: #ty,
                    }
                });
                Some(quote! {

                    #[derive(serde::Serialize)]
                    struct __Query <#lifetime> {
                        #(#fields)*
                    }

                })
            }
            true => None,
        };
        tokens.extend(stream);

        // set query
        let stream = match metas.is_empty() {
            true => None,
            false => {
                let fields = metas.iter().map(|meta| {
                    let ident = &meta.ident;
                    quote! { #ident }
                });
                Some(quote! {
                    let __request = __request.query(&__Query { #(#fields),* });
                })
            }
        };
        tokens.extend(stream);
    }
}
