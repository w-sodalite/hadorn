use crate::symbol::Symbol;
use proc_macro2::Ident;
use quote::format_ident;
use std::marker::PhantomData;
use syn::parse::{Parse, ParseStream};
use syn::{
    parenthesized, parse_quote, AttrStyle, Attribute, Error, Expr, Lit, LitStr, MetaNameValue,
    Path, Token,
};

macro_rules! arg {
    ($name:ident,$ty:ty) => {
        pub struct $name<T> {
            pub value: $ty,
            _p: PhantomData<T>
        }

        impl<T: Parse> Parse for $name<T> {
            fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
                let _ = input.parse::<T>()?;
                let _ = input.parse::<Token![=]>()?;
                let value = input.parse()?;
                Ok(Self {
                    value,
                    _p: PhantomData,
                })
            }
        }
    };
}

arg!(ExprArg, Expr);
arg!(StrArg, LitStr);

impl<T> ExprArg<T> {
    pub fn require_ident(&self) -> syn::Result<Ident> {
        get_expr_ident(&self.value).ok_or_else(|| {
            Error::new_spanned(&self.value, "only identifier or string literal allowed")
        })
    }
}

pub struct StrTuple(pub LitStr, pub LitStr);

impl Parse for StrTuple {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = parenthesized!(content in input);
        let items = content.parse_terminated(|stream| stream.parse::<LitStr>(), Token![,])?;
        if items.len() != 2 {
            return Err(input.error("expected exactly two items"));
        }
        let name = items.get(0).unwrap().clone();
        let value = items.get(1).unwrap().clone();
        Ok(Self(name, value))
    }
}

pub fn get_expr_ident(expr: &Expr) -> Option<Ident> {
    match expr {
        Expr::Lit(lit) => {
            if let Lit::Str(lit) = &lit.lit {
                Some(format_ident!("{}", lit.value()))
            } else {
                None
            }
        }
        Expr::Path(path) => path.path.get_ident().cloned(),
        _ => None,
    }
}

pub fn http_mod_path() -> Path {
    parse_quote!(hadorn::__http)
}

pub fn reqwest_mod_path() -> Path {
    parse_quote!(hadorn::__reqwest)
}

pub fn get_name_value(attrs: &[Attribute], name: Symbol) -> Option<&MetaNameValue> {
    attrs
        .iter()
        .filter(|attr| attr.style == AttrStyle::Outer)
        .filter(|attr| attr.path() == name)
        .flat_map(|attr| attr.meta.require_name_value())
        .next()
}
