mod body;
mod headers;
mod path;
mod query;
mod request;
mod response;
mod url;

use crate::contract::body::BodyTokens;
use crate::contract::headers::HeaderTokens;
use crate::contract::query::QueryTokens;
use crate::contract::request::RequestTokens;
use crate::contract::response::ResponseTokens;
use crate::contract::url::UrlTokens;
use crate::meta::PatMetas;
use crate::util::{ExprArg, StrArg, StrTuple};
use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::quote;
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::{
    bracketed, parse_quote, Attribute, Error, FnArg, LitStr, Pat, Signature, Token, TraitItemFn,
};

#[derive(Default, Debug)]
pub(crate) struct Contract {
    pub method: Option<Ident>,
    pub path: Option<LitStr>,
    pub headers: Option<Vec<(LitStr, LitStr)>>,
    pub serialized: Option<Ident>,
    pub deserialized: Option<Ident>,
}

impl Parse for Contract {
    //noinspection DuplicatedCode
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut contract = Contract::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::path) {
                if contract.method.is_some() {
                    return Err(input.error("duplicate attribute `path`"));
                }
                let path = input.parse::<StrArg<kw::path>>()?;
                contract.path = Some(path.value);
            } else if lookahead.peek(kw::headers) {
                if contract.headers.is_some() {
                    return Err(input.error("duplicate attribute `headers`"));
                }
                let headers = input.parse::<Headers>()?;
                contract.headers = Some(headers.0)
            } else if lookahead.peek(kw::serialized) {
                if contract.serialized.is_some() {
                    return Err(input.error("duplicate attribute `serialized`"));
                }
                let serialized = input
                    .parse::<ExprArg<kw::serialized>>()
                    .and_then(|serialized| serialized.require_ident())?;
                contract.serialized = Some(serialized);
            } else if lookahead.peek(kw::deserialized) {
                if contract.deserialized.is_some() {
                    return Err(input.error("duplicate attribute `deserialized`"));
                }
                let deserialized = input
                    .parse::<ExprArg<kw::deserialized>>()
                    .and_then(|deserialized| deserialized.require_ident())?;
                contract.deserialized = Some(deserialized);
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                let _ = input.parse::<TokenTree>()?;
            }
        }
        Ok(contract)
    }
}

impl Contract {
    pub fn expand(self, item: TraitItemFn) -> syn::Result<TokenStream> {
        let Contract {
            method,
            path,
            headers,
            serialized,
            deserialized,
        } = self;
        let TraitItemFn {
            mut attrs,
            mut sig,
            default,
            ..
        } = item;

        if default.is_some() {
            return Err(Error::new_spanned(
                default,
                "hadorn macros can not be applied to default function",
            ));
        }
        if sig.asyncness.is_none() {
            return Err(Error::new_spanned(
                sig,
                "hadorn macros only can applied to async function",
            ));
        }
        if sig.receiver().is_none() {
            sig.inputs.insert(0, FnArg::Receiver(parse_quote!(&self)));
        }

        let method =
            method.ok_or_else(|| Error::new_spanned(&sig, "missing attribute: `method`"))?;
        let path = path
            .map(|path| path.value())
            .ok_or_else(|| Error::new_spanned(&sig, "missing attribute: `path`"))?;

        let metas = PatMetas::new(&sig.inputs)?;
        let url_tokens = UrlTokens::new(&metas, &path)?;
        let request_tokens = RequestTokens::new(method);
        let query_tokens = QueryTokens::new(&metas);
        let header_tokens = HeaderTokens::new(&metas, headers.unwrap_or_default());
        let body_tokens = BodyTokens::new(&metas, &attrs, serialized)?;
        let response_tokens = ResponseTokens::new(&attrs, deserialized)?;

        reformat(&mut sig, &mut attrs, &metas);

        Ok(quote! {
            #(#attrs)*
            #sig {
                #url_tokens
                #request_tokens
                #query_tokens
                #header_tokens
                #body_tokens
                #response_tokens
            }
        })
    }
}

fn reformat(sig: &mut Signature, attrs: &mut Vec<Attribute>, metas: &PatMetas) {
    reformat_optional_input(sig, metas);
    reformat_input_attrs(sig);
    reformat_fn_attrs(attrs);
}

fn reformat_optional_input(sig: &mut Signature, metas: &PatMetas) {
    let options = metas
        .iter()
        .filter(|meta| meta.optional)
        .map(|meta| &meta.ident)
        .collect::<HashSet<_>>();
    sig.inputs
        .iter_mut()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat) => Some(pat),
        })
        .filter_map(|pat| match pat.pat.as_ref() {
            Pat::Ident(ident) => {
                if options.contains(&ident.ident) {
                    Some(pat)
                } else {
                    None
                }
            }
            _ => None,
        })
        .for_each(|pat| {
            let ty = pat.ty.clone();
            pat.ty = parse_quote! { Option<#ty> };
        });
}

fn reformat_input_attrs(sig: &mut Signature) {
    sig.inputs.iter_mut().for_each(|arg| match arg {
        FnArg::Receiver(r) => r.attrs.clear(),
        FnArg::Typed(t) => t.attrs.clear(),
    });
}

fn reformat_fn_attrs(attrs: &mut Vec<Attribute>) {
    attrs.clear()
}

#[derive(Default)]
struct Headers(Vec<(LitStr, LitStr)>);

impl Parse for Headers {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<kw::headers>()?;
        let _ = input.parse::<Token![=]>()?;
        let content;
        let _ = bracketed!(content in input);
        let headers = content
            .parse_terminated(StrTuple::parse, Token![,])?
            .into_iter()
            .map(|header| (header.0, header.1))
            .collect::<Vec<_>>();
        Ok(Self(headers))
    }
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(path);
    custom_keyword!(headers);
    custom_keyword!(serialized);
    custom_keyword!(deserialized);
}
