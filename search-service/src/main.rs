//! A small axum service in front of Loams: `GET /search?q=refund&tenant=a`
//! runs a hybrid search (vector + BM25 fused with RRF, optional tenant filter) through the SDK's
//! transport and answers JSON.
//!
//! Run `quickstart` first, which creates and fills `examples-rust/kb`.
//!
//! ```sh
//! LOAMS_ENDPOINT=http://127.0.0.1:8080 cargo run -p search-service
//! curl 'localhost:3737/search?q=refund&v=1,0,0&tenant=a'
//! ```

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use loams::TransportOptions;
use loams_proto::loams::collection::v1::{QueryServiceClient, SearchRequest};
use serde::Deserialize;
use serde_json::{Value, json};

const NAMESPACE: &str = "examples-rust";
const COLLECTION: &str = "kb";

type Client = QueryServiceClient<connectrpc::client::HttpClient>;

#[derive(Deserialize)]
struct Params {
    q: String,
    /// A comma-separated 3-d query vector; the example's embeddings are fake.
    v: Option<String>,
    tenant: Option<String>,
    limit: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint =
        std::env::var("LOAMS_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
    let options = TransportOptions::new(endpoint);
    let client: Client = QueryServiceClient::new(loams::transport(&options)?, options.config()?);

    let app = Router::new()
        .route("/search", get(search))
        .route("/healthz", get(|| async { "ok" }))
        .with_state(Arc::new(client));
    let listen = std::env::var("LISTEN").unwrap_or_else(|_| "127.0.0.1:3737".into());
    let listener = tokio::net::TcpListener::bind(&listen).await?;
    println!("listening on http://{listen}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn search(
    State(client): State<Arc<Client>>,
    Query(params): Query<Params>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let vector: Vec<f32> = params
        .v
        .as_deref()
        .unwrap_or("1,0,0")
        .split(',')
        .filter_map(|x| x.trim().parse().ok())
        .collect();
    let mut body = json!({
        "namespace": NAMESPACE,
        "collection": COLLECTION,
        "retrievers": [
            {"vector": {"field": "embedding", "query": vector, "k": 20}},
            {"text": {"query": {"match": {"field": "body", "text": params.q}}, "k": 20}}
        ],
        "fusion": {"rrf": {"k": 60}},
        "limit": params.limit.unwrap_or(5),
    });
    if let Some(tenant) = params.tenant {
        body["filter"] = json!({"term": {"field": "tenant", "value": tenant}});
    }
    let bad = |e: String| (StatusCode::INTERNAL_SERVER_ERROR, e);
    let request: SearchRequest = serde_json::from_value(body).map_err(|e| bad(e.to_string()))?;
    let response = client
        .search(request)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("{e:?}")))?
        .into_owned();
    let hits: Vec<Value> = response
        .hits
        .iter()
        .map(|h| {
            let source = serde_json::to_value(&h.source).unwrap_or(Value::Null);
            json!({"id": serde_json::to_value(&h.pk).unwrap_or(Value::Null), "score": h.score, "source": source})
        })
        .collect();
    Ok(Json(json!({ "hits": hits })))
}
