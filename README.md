# Hadorn

[![Crates.io][crates-badge]][crates-url]
[![Apache licensed][apache-badge]][apache-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/hadorn.svg
[crates-url]: https://crates.io/crates/hadorn
[apache-badge]: https://img.shields.io/badge/license-Aapche-blue.svg
[apache-url]: LICENSE
[actions-badge]: https://github.com/w-sodalite/hadorn/workflows/CI/badge.svg
[actions-url]: https://github.com/w-sodalite/hadorn/actions?query=workflow%3ACI


A type-safe HTTP client for Rust, inspire by [retrofit](https://github.com/square/retrofit).

## Example

your `Cargo.toml` could look like this:

```toml
[dependencies]
hadorn = { version = "0.1" }
reqwest = { version = "0.12", features = ["json"] }
```

And then the code:

```rust

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
        #[query = "q"] keyword: &str,
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
    let respond = client.list(1, 5, Some("reqwest")).await.unwrap();
    println!("{:#?}", respond);
}

```

## Macro

- `hadorn`

    > define a grouped apis `client`、`serialized`、`deserialized`.

    - `client`: Generate the `client` struct name, default is the trait name append `Client`
  
    - `serialized`: The trait all apis default serialize type
      - Json  => `request.json(...)`
      - FormData => `request.form(...)`
      - Multipart => `request.multipart(...)`
      - no set =>  `request.body(...)`

    - `deserialized`: The trait all apis default deserialize type
      - Text => `response.text()`
      - Json => `response.json()`
      - Bytes => `response.bytes()`
      - no set => `response`

- `get` | `post` | `put` | `delete` | `head` | `option` | `trace`

    > define a http request `method`、`path`、`headers`、`serialized`、`deserialzed`.

    - `path`: request path
    - `headers`: request headers, examples: `headers = [("content-type", "application/json")]`
    - `serialized`: same of `hadorn`, priority is higher.
    - `deserialized`: same of `hadorn`, priority is higher.


- `#[path]` | `#[query]` | `#[header]` | `#[body]`
  
    > `#[path]`、`#[query]`、`#[header]` can set a literal to rename the argument name: `#[path = "version"]`, if the request param name not equals arument name. 

    > `#[body]` mark the argument is request body argument, only appear once.



## Notice

`hadorn` current only supported `reqwest` library, The support for other HTTP client libraries will be added
subsequently.

## License

This project is licensed under the [Apache 2.0](./LICENSE)