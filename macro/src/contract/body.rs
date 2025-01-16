use crate::meta::{Kind, PatMetas};
use crate::util::{get_expr_ident, get_name_value};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, Error};

pub struct BodyTokens {
    body: Option<Ident>,
    serialized: Option<Ident>,
}

impl BodyTokens {
    pub fn new(
        metas: &PatMetas,
        attrs: &[Attribute],
        serialized: Option<Ident>,
    ) -> syn::Result<Self> {
        let body = metas
            .iter()
            .find(|meta| meta.kind == Kind::Body)
            .map(|meta| &meta.ident)
            .cloned();
        let serialized = get_serialized(attrs).map(|s| s.or(serialized))?;
        Ok(Self { body, serialized })
    }
}

impl ToTokens for BodyTokens {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stream = self.body.as_ref().map(|body| match &self.serialized {
            Some(serialized) => {
                if serialized == symbol::JSON {
                    quote! {
                        let __request = __request.json(&#body);
                    }
                } else if serialized == symbol::FORM_DATA {
                    quote! {
                        let __request = __request.form_data(&#body);
                    }
                } else if serialized == symbol::MULTIPART {
                    quote! {
                        let __request = __request.multipart(&#body);
                    }
                } else {
                    quote! {
                        let __request = __request.body(&#body);
                    }
                }
            }
            None => quote! {
                quote! {
                    let __request = __request.body(&#body);
                }
            },
        });
        tokens.extend(stream);
    }
}

fn get_serialized(attrs: &[Attribute]) -> syn::Result<Option<Ident>> {
    match get_name_value(attrs, symbol::SERIALIZED)
        .cloned()
        .map(|name_value| name_value.value)
    {
        None => Ok(None),
        Some(expr) => get_expr_ident(&expr)
            .ok_or_else(|| Error::new(Span::call_site(), "invalid attribute: `serialized`"))
            .map(Some),
    }
}

mod symbol {
    use crate::symbol::Symbol;

    pub const SERIALIZED: Symbol = Symbol("serialized");
    pub const JSON: Symbol = Symbol("Json");
    pub const FORM_DATA: Symbol = Symbol("FormData");
    pub const MULTIPART: Symbol = Symbol("Multipart");
}
