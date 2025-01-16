use crate::util::{http_mod_path, reqwest_mod_path, ExprArg};
use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, ItemTrait, Token, TraitItem, TypeParamBound};

#[derive(Debug, Default)]
pub struct Resource {
    client: Option<Ident>,
    serialized: Option<Ident>,
    deserialized: Option<Ident>,
}

impl Parse for Resource {
    //noinspection DuplicatedCode
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut resource = Resource::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::client) {
                if resource.client.is_some() {
                    return Err(input.error("duplicate attribute `client`"));
                }
                let client = input
                    .parse::<ExprArg<kw::client>>()
                    .and_then(|client| client.require_ident())?;
                resource.client = Some(client);
            } else if lookahead.peek(kw::serialized) {
                if resource.serialized.is_some() {
                    return Err(input.error("duplicate attribute `serialized`"));
                }
                let serialize = input
                    .parse::<ExprArg<kw::serialized>>()
                    .and_then(|serialize| serialize.require_ident())?;
                resource.serialized = Some(serialize);
            } else if lookahead.peek(kw::deserialized) {
                if resource.deserialized.is_some() {
                    return Err(input.error("duplicate attribute `deserialized`"));
                }
                let deserialize = input
                    .parse::<ExprArg<kw::deserialized>>()
                    .and_then(|deserialize| deserialize.require_ident())?;
                resource.deserialized = Some(deserialize);
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                let _ = input.parse::<TokenTree>()?;
            }
        }
        Ok(resource)
    }
}

impl Resource {
    pub fn expand(self, mut item_trait: ItemTrait) -> syn::Result<TokenStream> {
        let vis = &item_trait.vis;
        let name = &item_trait.ident;
        let client = self.client.unwrap_or(format_ident!("{}Client", name));
        let serialized = self.serialized;
        let deserialized = self.deserialized;

        // insert [Hadorn] super trait
        item_trait
            .supertraits
            .push(TypeParamBound::Trait(parse_quote! { hadorn::Hadorn }));

        // add serialized and deserialized attribute
        item_trait.items.iter_mut().for_each(|item| {
            if let TraitItem::Fn(item_fn) = item {
                if let Some(serialized) = &serialized {
                    item_fn
                        .attrs
                        .push(parse_quote!(#[serialized = #serialized]));
                }
                if let Some(deserialized) = &deserialized {
                    item_fn
                        .attrs
                        .push(parse_quote!(#[deserialized = #deserialized]));
                }
            }
        });

        let http_mod = http_mod_path();
        let reqwest_mod = reqwest_mod_path();

        // struct and block
        let struct_block = quote! {

            #[derive(Clone, Default)]
            #vis struct #client{
                #[doc = "reqwest client"]
                client: #reqwest_mod::Client,

                #[doc = "base url"]
                base_url: Option<String>,

                #[doc = "default http headers"]
                default_headers: Option<#http_mod::HeaderMap>
            }

            impl #client {

                #[doc = "construct with reqwest client"]
                pub fn new(client: #reqwest_mod::Client) -> Self {
                    Self{
                        client,
                        base_url: None,
                        default_headers: None
                    }
                }

                #[doc = "set base url for the client"]
                pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
                    self.base_url = Some(base_url.into());
                    self
                }

                #[doc = "set default http headers for the client"]
                pub fn with_default_headers(mut self, default_headers: #http_mod::HeaderMap) -> Self {
                    self.default_headers = Some(default_headers);
                    self
                }
            }
        };

        // impl api trait for struct
        let impl_api_trait = quote! {
            impl #name for #client {}
        };

        // impl hadorn trait for struct
        let impl_hadorn_trait = quote! {
            impl hadorn::Hadorn for #client {

                fn client(&self) -> &#reqwest_mod::Client {
                    &self.client
                }

                fn base_url(&self) -> Option<&str> {
                    self.base_url.as_deref()
                }

                fn default_headers(&self) -> Option<&#http_mod::HeaderMap> {
                    self.default_headers.as_ref()
                }
            }
        };

        Ok(quote! {
            #item_trait

            #struct_block

            #impl_api_trait

            #impl_hadorn_trait
        })
    }
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(client);
    custom_keyword!(serialized);
    custom_keyword!(deserialized);
}
