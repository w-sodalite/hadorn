#![allow(unused)]

use hadorn::{get, hadorn};
use http::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Result};
use serde::Deserialize;

#[hadorn(
    serialized = Json,
    deserialized = Json
)]
trait Crates {
    #[get(path = "/crates")]
    async fn list(
        #[query] page: usize,
        #[query = "per_page"] page_size: usize,
        #[optional]
        #[query = "q"]
        keyword: &str,
    ) -> Result<QueryCrateRespond>;
}

#[derive(Debug, Deserialize)]
struct QueryCrateRespond {
    crates: Vec<Crate>,
    meta: Meta,
}

#[derive(Debug, Deserialize)]
struct Crate {
    id: String,
    created_at: String,
    default_version: String,
    description: String,
    documentation: Option<String>,
    homepage: Option<String>,
    downloads: u64,
}

#[derive(Debug, Deserialize)]
struct Meta {
    next_page: Option<String>,
    prev_page: Option<String>,
    total: u64,
}

#[tokio::test]
async fn call_list() {
    let client = CratesClient::new(Client::new())
        .with_base_url("https://crates.io/api/v1")
        .with_default_headers(HeaderMap::from_iter([(
            HeaderName::from_static("user-agent"),
            HeaderValue::from_static("hadorn-rs"),
        )]));
    let respond = client.list(1, 5, Some("reqwest")).await;
    assert!(respond.is_ok());
}
