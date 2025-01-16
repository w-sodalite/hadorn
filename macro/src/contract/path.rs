use crate::meta::{Kind, PatMetas};
use proc_macro2::Span;
use std::collections::HashSet;
use syn::Error;

pub struct PathParams(Vec<String>);

impl PathParams {
    pub fn new(path: &str) -> Self {
        let start = path.find("://").unwrap_or(0);
        let end = path.find("?").unwrap_or(path.len());
        let params = path[start..end]
            .split('/')
            .filter(|item| item.starts_with('<') && item.ends_with('>'))
            .flat_map(|item| item.strip_prefix('<'))
            .flat_map(|item| item.strip_suffix('>'))
            .map(|item| item.to_string())
            .collect();
        Self(params)
    }

    pub fn check(self, metas: &PatMetas) -> syn::Result<Self> {
        self.check_undefined_params(metas)
            .and_then(|_| self.check_missing_params(metas))
            .map(|_| self)
    }

    fn check_undefined_params(&self, metas: &PatMetas) -> syn::Result<()> {
        let metas = metas
            .iter()
            .filter(|meta| meta.kind == Kind::Path)
            .map(|meta| match &meta.rename {
                Some(rename) => rename.value(),
                None => meta.ident.to_string(),
            })
            .collect::<HashSet<_>>();
        self.0
            .iter()
            .try_fold((), |_, param| match metas.contains(param) {
                true => Ok(()),
                false => Err(Error::new(
                    Span::call_site(),
                    format!("undefined path param: `{}`", param),
                )),
            })
    }

    fn check_missing_params(&self, metas: &PatMetas) -> syn::Result<()> {
        metas
            .iter()
            .filter(|meta| meta.kind == Kind::Path)
            .try_fold((), |_, meta| {
                let param = match &meta.rename {
                    None => meta.ident.to_string(),
                    Some(rename) => rename.value(),
                };
                match self.0.contains(&param) {
                    true => Ok(()),
                    false => Err(Error::new(
                        Span::call_site(),
                        format!("missing path param: `{}`", param),
                    )),
                }
            })
    }

    pub fn get_format_pattern(path: &str) -> String {
        let mut pattern = String::with_capacity(path.len());
        let start = path.find("://").unwrap_or(0);
        let end = path.find("?").unwrap_or(path.len());
        if start > 0 {
            pattern.push_str(&path[..start]);
        }
        path[start..end]
            .split('/')
            .filter(|item| !item.is_empty())
            .for_each(|item| {
                pattern.push('/');
                if item.starts_with('<') && item.ends_with('>') {
                    pattern.push_str("{}");
                } else {
                    pattern.push_str(item);
                }
            });
        if end < path.len() {
            pattern.push_str(&path[end..]);
        }
        pattern
    }
}
