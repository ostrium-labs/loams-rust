//! Axum adapter that runs a high-level SDK hybrid search for `GET /search`.
//! Run `quickstart` first to seed `examples-rust/kb`.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use loams::builders::{Filter, Fusion, Search, TextRetriever, VectorOptions};
use loams::{CallOptionsOverrides, Loams};
use serde::Deserialize;
use serde_json::{Value, json};

const COLLECTION: &str = "kb";
type Client = Loams;

#[derive(Deserialize)]
struct Params {
    q: String,
    /// A comma-separated 3-d query vector; the sample embeddings are illustrative.
    v: Option<String>,
    tenant: Option<String>,
    limit: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Loams::from_env()?.with_namespace("examples-rust");
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
    let vector: Vec<f32> = params.v.as_deref().unwrap_or("1,0,0")
        .split(',').map(str::trim).map(str::parse).collect::<Result<_, _>>()
        .map_err(|e: std::num::ParseFloatError| (StatusCode::BAD_REQUEST, e.to_string()))?;
    if vector.len() != 3 {
        return Err((StatusCode::BAD_REQUEST, "v must contain exactly three numbers".into()));
    }
    let mut request = Search::on(COLLECTION)
        .text(TextRetriever::fields(&params.q, ["body"]))
        .vector("embedding", &vector, VectorOptions::k(20))
        .fuse(Fusion::rrf(Some(60)))
        .limit(params.limit.unwrap_or(5));
    if let Some(tenant) = params.tenant {
        request = request.filter(Filter::eq("tenant", tenant));
    }
    let response = client.search().query(request.build().map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
        CallOptionsOverrides::new()).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("{e:?}")))?;
    let hits: Vec<Value> = response.hits.iter().map(|h| {
        json!({"id": serde_json::to_value(&h.pk).unwrap_or(Value::Null),
               "score": h.score,
               "source": serde_json::to_value(&h.source).unwrap_or(Value::Null)})
    }).collect();
    Ok(Json(json!({"hits": hits})))
}
