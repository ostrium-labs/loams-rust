//! Quickstart: connect, create a collection, upsert documents, run a hybrid
//! search (vector + BM25 + filter) and print the hits.
//!
//! ```sh
//! LOAMS_ENDPOINT=http://127.0.0.1:8080 cargo run -p quickstart
//! ```
//!
//! Everything lives in the `examples-rust` namespace.

use loams::instance_v1::GetInstanceRequest;
use loams::{CallOptionsOverrides, Loams, TransportOptions};
use loams_proto::loams::collection::v1::{
    CollectionServiceClient, CreateCollectionRequest, CreateNamespaceRequest, DocumentServiceClient,
    DropCollectionRequest, NamespaceServiceClient, QueryServiceClient, SearchRequest,
    WriteDocumentsRequest,
};
use serde_json::{from_value, json};

const NAMESPACE: &str = "examples-rust";
const COLLECTION: &str = "kb";

type BoxError = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let endpoint =
        std::env::var("LOAMS_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
    let options = TransportOptions::new(endpoint.clone());

    // The SDK client: what is this instance, and what does it serve?
    let loams = Loams::connect(options.clone())?;
    let info = loams
        .instance()
        .get_instance(GetInstanceRequest::default(), CallOptionsOverrides::new())
        .await?;
    println!("connected to {} {} ({endpoint})", info.name, info.server_version);

    // Demonstrate the generated `loams-proto` clients over the SDK transport.
    // SDK collection handles and builders are also available.
    let transport = loams::transport(&options)?;
    let config = options.config()?;
    let namespaces = NamespaceServiceClient::new(transport.clone(), config.clone());
    let collections = CollectionServiceClient::new(transport.clone(), config.clone());
    let documents = DocumentServiceClient::new(transport.clone(), config.clone());
    let query = QueryServiceClient::new(transport, config);

    // Create the namespace (an existing one is fine) and a fresh collection.
    let _ = namespaces
        .create_namespace(from_value::<CreateNamespaceRequest>(json!({ "namespace": NAMESPACE }))?)
        .await;
    let _ = collections
        .drop_collection(from_value::<DropCollectionRequest>(
            json!({ "namespace": NAMESPACE, "collection": COLLECTION }),
        )?)
        .await;
    collections
        .create_collection(from_value::<CreateCollectionRequest>(json!({
            "namespace": NAMESPACE,
            "name": COLLECTION,
            "schema": {
                "fields": [
                    {"name": "body", "source_path": "body",
                     "kind": {"text": {"analyzer": "standard", "positions": true}},
                     "indexed": true, "fast": false},
                    {"name": "tenant", "source_path": "tenant", "kind": "keyword",
                     "indexed": true, "fast": true}
                ],
                "vectors": [{"name": "embedding", "dim": 3, "distance": "cosine"}],
                "sparse_vectors": [], "dynamic": "ignore", "max_fields": 1000
            }
        }))?)
        .await?;
    println!("created {NAMESPACE}/{COLLECTION}");

    // Upsert three documents with a vector each.
    documents
        .write_documents(from_value::<WriteDocumentsRequest>(json!({
            "namespace": NAMESPACE,
            "collection": COLLECTION,
            "ops": [
                {"upsert": {"id": {"uint": "1"}, "source": {"body": "refund policy", "tenant": "a"},
                            "vectors": {"embedding": [1.0, 0.0, 0.0]}}},
                {"upsert": {"id": {"uint": "2"}, "source": {"body": "shipping times", "tenant": "a"},
                            "vectors": {"embedding": [0.9, 0.1, 0.0]}}},
                {"upsert": {"id": {"uint": "3"}, "source": {"body": "refund window", "tenant": "b"},
                            "vectors": {"embedding": [0.0, 0.0, 1.0]}}}
            ]
        }))?)
        .await?;
    println!("upserted 3 documents");

    // Hybrid search: vector kNN + BM25, fused with RRF, filtered to tenant "a".
    let hits = query
        .search(from_value::<SearchRequest>(json!({
            "namespace": NAMESPACE,
            "collection": COLLECTION,
            "retrievers": [
                {"vector": {"field": "embedding", "query": [1.0, 0.0, 0.0], "k": 10}},
                {"text": {"query": {"match": {"field": "body", "text": "refund"}}, "k": 10}}
            ],
            "filter": {"term": {"field": "tenant", "value": "a"}},
            "fusion": {"rrf": {"k": 60}},
            "limit": 3
        }))?)
        .await?
        .into_owned();
    for hit in &hits.hits {
        let source = serde_json::to_value(&hit.source)?;
        println!("{:.4}  {}", hit.score, source["body"].as_str().unwrap_or("?"));
    }
    Ok(())
}
