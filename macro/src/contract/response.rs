use crate::util::{get_expr_ident, get_name_value};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, Error};

pub struct ResponseTokens {
    deserialized: Option<Ident>,
}

impl ResponseTokens {
    pub fn new(attrs: &[Attribute], deserialized: Option<Ident>) -> syn::Result<Self> {
        let deserialized = get_deserialized(&attrs).map(|d| d.or(deserialized))?;
        Ok(Self { deserialized })
    }
}

impl ToTokens for ResponseTokens {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stream = quote! {
            let __response = __request.send().await.and_then(|response| response.error_for_status())?;
        };
        tokens.extend(stream);

        let stream = match self.deserialized.as_ref() {
            Some(deserialize) => {
                if deserialize == symbol::JSON {
                    quote! {
                        __response.json().await
                    }
                } else if deserialize == symbol::TEXT {
                    quote! {
                        __response.text().await
                    }
                } else if deserialize == symbol::BYTES {
                    quote! {
                        __response.bytes().await
                    }
                } else {
                    quote! {
                        Ok(__response)
                    }
                }
            }
            None => quote! { Ok(__response) },
        };
        tokens.extend(stream);
    }
}

fn get_deserialized(attrs: &[Attribute]) -> syn::Result<Option<Ident>> {
    match get_name_value(attrs, symbol::DESERIALIZED)
        .cloned()
        .map(|name_value| name_value.value)
    {
        None => Ok(None),
        Some(expr) => get_expr_ident(&expr)
            .ok_or_else(|| Error::new(Span::call_site(), "invalid attribute: `deserialized`"))
            .map(Some),
    }
}

mod symbol {
    use crate::symbol::Symbol;

    pub const DESERIALIZED: Symbol = Symbol("deserialized");
    pub const JSON: Symbol = Symbol("Json");
    pub const TEXT: Symbol = Symbol("Text");
    pub const BYTES: Symbol = Symbol("Bytes");
}
