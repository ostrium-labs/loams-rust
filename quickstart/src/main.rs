//! Runnable tour of the published 0.1.2 SDK modules. All writes are confined
//! to the `examples-rust` namespace and use stable IDs / producer keys.
//!
//! ```sh
//! LOAMS_ENDPOINT=http://127.0.0.1:8080 cargo run -p quickstart
//! ```

use futures::StreamExt;
use loams::builders::{DocId, Filter, Fusion, Schema, Search, Sort, TextRetriever, VectorOptions};
use loams::collection_v1::{CreateCollectionRequest, CreateNamespaceRequest};
use loams::graph_v1::{CreateGraphRequest, ExecuteRequest};
use loams::instance_v1::GetInstanceRequest;
use loams::stream_v1::{CreateStreamRequest, FetchRequest, ProduceRequest, Record};
use loams::{CallOptionsOverrides, Document, ErrorKind, Loams, LoamsError};
use serde_json::json;

const NAMESPACE: &str = "examples-rust";
const COLLECTION: &str = "kb";
type BoxError = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let loams = Loams::from_env()?.with_namespace(NAMESPACE).session_consistency(true);
    let no_options = CallOptionsOverrides::new;

    // Instance and system modules: no-auth discovery, API compatibility and
    // optional-feature detection.
    let info = loams.instance().get_instance(GetInstanceRequest::default(), no_options()).await?;
    println!("connected to {} {}", info.name, info.server_version);
    let version = loams.system().version().await?;
    println!("SDK/server compatible: {}", version.compatible);
    let catalogue = loams.system().catalogue().await?;
    println!("{} served packages", catalogue.served.len());

    // Namespace and collection modules. Creation is idempotent; no collection
    // is dropped or recreated on a rerun.
    let result = loams.namespaces().create(
        CreateNamespaceRequest { namespace: NAMESPACE.into(), ..Default::default() }, no_options()).await;
    ignore_existing(result.map(|_| ()))?;
    let schema = Schema::new().text("title").text("body").keyword("tenant").i64("n")
        .vector("embedding", 3);
    let result = loams.collections().create(CreateCollectionRequest {
        name: COLLECTION.into(), schema: buffa::MessageField::some(schema.to_struct()?), ..Default::default()
    }, no_options()).await;
    ignore_existing(result.map(|_| ()))?;

    // Documents module through the collection handle: upsert, get, patch,
    // delete, count, and scroll. Stable IDs make the seed repeatable.
    let kb = loams.collection(COLLECTION, None);
    kb.upsert(&[
        Document::new(1u64).source(json!({"title":"Refunds","body":"refund policy for damaged items","tenant":"a","n":1}))
            .vector("embedding", vec![1.0, 0.0, 0.0]),
        Document::new(2u64).source(json!({"title":"Shipping","body":"shipping times and carriers","tenant":"a","n":2}))
            .vector("embedding", vec![0.0, 1.0, 0.0]),
        Document::new(3u64).source(json!({"title":"Refund window","body":"refund window for digital goods","tenant":"b","n":3}))
            .vector("embedding", vec![0.0, 0.0, 1.0]),
    ], no_options()).await?;
    println!("read back {} document(s)", kb.get(&[DocId::from(1u64), DocId::from(2u64)], no_options()).await?.documents.len());
    let count = kb.count(Some(&Filter::eq("tenant", "a")), no_options()).await?;
    println!("tenant a count: {:?}", count.count);
    let scroll = kb.scroll_all(None, Some(10), no_options());
    futures::pin_mut!(scroll);
    while let Some(doc) = scroll.next().await { println!("scroll item: {:?}", doc?.id); }

    // Search and vector modules. The collection handle also exposes these
    // builders; the module calls demonstrate their direct entry points.
    let text_request = Search::on(COLLECTION).text(TextRetriever::fields("refund", ["body"]))
        .filter(Filter::eq("tenant", "a")).sort([Sort::score(None), Sort::pk(None)]).limit(5).build()?;
    let text = loams.search().query(text_request, no_options()).await?;
    let vector_request = Search::on(COLLECTION).vector("embedding", &[1.0, 0.0, 0.0], VectorOptions::k(5)).build()?;
    let vector = loams.vector().search(vector_request, no_options()).await?;
    let hybrid = kb.search(kb.query().text(TextRetriever::fields("refund", ["body"]))
        .vector("embedding", &[1.0, 0.0, 0.0], VectorOptions::k(5))
        .fuse(Fusion::rrf(Some(60))).limit(3), no_options()).await?;
    println!("text {}, vector {}, hybrid {} hit(s)", text.hits.len(), vector.hits.len(), hybrid.hits.len());

    // SQL module: records are returned as column-name/value maps.
    let rows = loams.sql().records("SELECT title, tenant FROM kb WHERE n > $1 ORDER BY n",
        vec![json!(1)], no_options()).await?;
    println!("SQL rows: {}", rows.len());

    // Streams module: a stable producer sequence makes a rerun a duplicate
    // response instead of a second append. Fetch just the returned offsets.
    loams.streams().create(CreateStreamRequest { name: "sdk-tour-events".into(), partitions: 1,
        ..Default::default() }, no_options()).await?;
    let produced = loams.streams().produce(ProduceRequest { stream: "sdk-tour-events".into(),
        records: vec![Record { value: Some(br#"{"kind":"seeded","id":1}"#.to_vec()), ..Default::default() }],
        producer_id: Some("examples-rust-sdk-tour".into()), producer_seq: Some(1), ..Default::default() },
        no_options()).await?;
    let mut events = loams.subscribe(FetchRequest { stream: "sdk-tour-events".into(), offset: produced.base_offset,
        ..Default::default() }, no_options());
    if let Some(event) = events.next().await { println!("stream event at {}", event?.offset); }

    // Graph is experimental and gated by the cached instance catalogue.
    if loams.system().available("graph").await? {
        let create = loams.graph().create(CreateGraphRequest { name: "examples_rust_sdk_tour".into(),
            ..Default::default() }, no_options()).await;
        ignore_existing(create.map(|_| ()))?;
        let rows = loams.graph().query(ExecuteRequest { graph: "examples_rust_sdk_tour".into(),
            statement: "RETURN 1 AS ready".into(), read_only: true, ..Default::default() }, no_options()).await?;
        println!("graph query returned {} row(s)", rows.rows.as_ref().map_or(0, |r| r.rows.len()));
    } else { println!("graph is not served by this instance"); }

    Ok(())
}

fn ignore_existing<T>(result: Result<T, LoamsError>) -> Result<(), LoamsError> {
    match result {
        Err(error) if error.kind == ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error),
        Ok(_) => Ok(()),
    }
}
