#[doc = include_str!("../README.md")]
pub trait Hadorn {
    ///
    /// reqwest client
    ///
    fn client(&self) -> &reqwest::Client;

    ///
    /// the request base url
    ///
    fn base_url(&self) -> Option<&str>;

    ///
    /// the request default headers
    ///
    fn default_headers(&self) -> Option<&http::HeaderMap>;
}

// export hadorn macro
pub use hadorn_macro::*;

#[doc(hidden)]
pub mod __http {
    pub use http::*;
}

#[doc(hidden)]
pub mod __reqwest {
    pub use reqwest::*;
}
