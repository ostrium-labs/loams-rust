//! SQL over a collection: plain `SELECT`s, then the hybrid table functions
//! `vector_search`, `text_search` and `rrf`.
//!
//! Run `quickstart` first, which creates and fills `examples-rust/kb`.
//!
//! ```sh
//! LOAMS_ENDPOINT=http://127.0.0.1:18080 cargo run -p sql
//! ```
//!
//! The 0.1 Rust SDK has no SQL call; SQL is a JSON POST to the native API's
//! `/v1/namespaces/{ns}/sql` route. The SDK is used to check the instance.

use loams::instance_v1::GetInstanceRequest;
use loams::{CallOptionsOverrides, Loams};
use serde_json::{Value, json};

const NAMESPACE: &str = "examples-rust";

type BoxError = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let endpoint =
        std::env::var("LOAMS_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
    let info = Loams::to(&endpoint)?
        .instance()
        .get_instance(GetInstanceRequest::default(), CallOptionsOverrides::new())
        .await?;
    println!("connected to {} {}", info.name, info.server_version);

    let http = reqwest::Client::new();
    let url = format!("{endpoint}/v1/namespaces/{NAMESPACE}/sql");

    run(&http, &url, "SELECT _id, body, tenant FROM kb ORDER BY _id").await?;
    run(
        &http,
        &url,
        "SELECT _id, body, _score FROM rrf(\
            vector_search('kb', [1.0, 0.0, 0.0], 'embedding', 10), \
            text_search('kb', 'refund', 'body', 10)) LIMIT 3",
    )
    .await
}

async fn run(http: &reqwest::Client, url: &str, query: &str) -> Result<(), BoxError> {
    println!("\n> {query}");
    let response = http.post(url).json(&json!({ "query": query })).send().await?;
    let status = response.status();
    let body: Value = response.json().await?;
    if !status.is_success() {
        return Err(format!("{status}: {body}").into());
    }
    let names: Vec<&str> = body["columns"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| c["name"].as_str())
        .collect();
    println!("{}", names.join(" | "));
    for row in body["rows"].as_array().into_iter().flatten() {
        let cells: Vec<String> = row
            .as_array()
            .into_iter()
            .flatten()
            .map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_owned))
            .collect();
        println!("{}", cells.join(" | "));
    }
    Ok(())
}
