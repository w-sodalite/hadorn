use crate::symbol::Symbol;
use syn::punctuated::Punctuated;
use syn::{
    AttrStyle, Attribute, Error, Expr, FnArg, Ident, Lit, LitStr, Meta, Pat, PatType, Token, Type,
};

pub struct PatMetas {
    metas: Vec<PatMeta>,
}

impl PatMetas {
    pub fn new(inputs: &Punctuated<FnArg, Token![,]>) -> syn::Result<Self> {
        inputs
            .iter()
            .flat_map(|arg| match arg {
                FnArg::Receiver(_) => None,
                FnArg::Typed(pat) => Some(pat),
            })
            .try_fold(vec![], |mut metas, pat| match PatMeta::try_from(pat) {
                Ok(meta) => {
                    metas.push(meta);
                    Ok(metas)
                }
                Err(e) => Err(e),
            })
            .map(|metas| Self { metas })
    }

    pub fn iter(&self) -> impl Iterator<Item = &PatMeta> {
        self.metas.iter()
    }
}

impl IntoIterator for PatMetas {
    type Item = PatMeta;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.metas.into_iter()
    }
}

#[derive(Debug)]
pub struct PatMeta {
    pub ident: Ident,
    pub ty: Box<Type>,
    pub kind: Kind,
    pub optional: bool,
    pub rename: Option<LitStr>,
    pub reference: bool,
}

impl TryFrom<&'_ PatType> for PatMeta {
    type Error = Error;

    fn try_from(pat: &PatType) -> Result<Self, Self::Error> {
        let PatType { attrs, pat, ty, .. } = pat;
        let ident = get_ident(pat)?;
        let optional = get_optional(attrs);
        let kind = get_kind(&ident, attrs)?;
        let rename = get_rename(&ident, attrs, &kind)?;
        let reference = matches!(ty.as_ref(), Type::Reference(_));
        Ok(Self {
            ident,
            ty: ty.clone(),
            optional,
            kind,
            rename,
            reference,
        })
    }
}

fn get_ident(pat: &Pat) -> syn::Result<Ident> {
    match pat {
        Pat::Ident(ident) => Ok(ident.ident.clone()),
        _ => Err(Error::new_spanned(pat, "expected identifier")),
    }
}

fn get_optional(attr: &[Attribute]) -> bool {
    attr.iter()
        .filter(|attr| attr.style == AttrStyle::Outer)
        .filter(|attr| attr.path() == symbol::OPTIONAL)
        .any(|attr| matches!(attr.meta, Meta::Path(_)))
}

fn get_kind(ident: &Ident, attr: &[Attribute]) -> syn::Result<Kind> {
    let is_symbol = |symbol: Symbol| {
        attr.iter()
            .filter(|attr| attr.style == AttrStyle::Outer)
            .any(|attr| attr.meta.path() == symbol)
    };
    if is_symbol(symbol::QUERY) {
        Ok(Kind::Query)
    } else if is_symbol(symbol::PATH) {
        Ok(Kind::Path)
    } else if is_symbol(symbol::BODY) {
        Ok(Kind::Body)
    } else if is_symbol(symbol::HEADER) {
        Ok(Kind::Header)
    } else {
        Err(Error::new_spanned(ident, "missing kind attribute"))
    }
}

fn get_rename(ident: &Ident, attrs: &[Attribute], kind: &Kind) -> syn::Result<Option<LitStr>> {
    let attr = attrs
        .iter()
        .filter(|attr| attr.style == AttrStyle::Outer)
        .find(|attr| attr.path() == kind.symbol());
    match attr {
        Some(attr) => match &attr.meta {
            Meta::Path(_) => Ok(None),
            Meta::List(_) => Err(Error::new_spanned(
                ident,
                "attribute `rename` unsupported meta list",
            )),
            Meta::NameValue(name_value) => match &name_value.value {
                Expr::Lit(lit) => match &lit.lit {
                    Lit::Str(lit) => Ok(Some(lit.clone())),
                    _ => Err(Error::new_spanned(
                        ident,
                        "attribute `rename` only supports literal strings",
                    )),
                },
                _ => Err(Error::new_spanned(
                    ident,
                    "attribute `rename` only supports literal strings",
                )),
            },
        },
        None => Err(Error::new_spanned(ident, "missing kind attribute")),
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Kind {
    Header,
    Query,
    Body,
    Path,
}

impl Kind {
    pub fn symbol(&self) -> Symbol {
        match self {
            Kind::Header => symbol::HEADER,
            Kind::Query => symbol::QUERY,
            Kind::Body => symbol::BODY,
            Kind::Path => symbol::PATH,
        }
    }
}

mod symbol {
    use crate::symbol::Symbol;

    pub const PATH: Symbol = Symbol("path");
    pub const BODY: Symbol = Symbol("body");
    pub const QUERY: Symbol = Symbol("query");
    pub const HEADER: Symbol = Symbol("header");
    pub const OPTIONAL: Symbol = Symbol("optional");
}
