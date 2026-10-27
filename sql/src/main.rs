//! SQL over `examples-rust/kb` using the SDK's `loams.sql` module.
//! Run `quickstart` first to create and seed the collection.

use loams::{CallOptionsOverrides, Loams};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loams = Loams::from_env()?.with_namespace("examples-rust").session_consistency(true);
    let options = CallOptionsOverrides::new();
    for (label, query, params) in [
        ("plain select", "SELECT _id, body, tenant FROM kb ORDER BY _id", vec![]),
        ("hybrid table functions", "SELECT _id, body, _score FROM rrf(vector_search('kb', [1.0, 0.0, 0.0], 'embedding', 10), text_search('kb', 'refund', 'body', 10)) LIMIT 3", vec![]),
        ("bound parameter", "SELECT tenant, count(*) AS n FROM kb WHERE n > $1 GROUP BY tenant ORDER BY tenant", vec![json!(1)]),
    ] {
        println!("\n-- {label}: {query}");
        for row in loams.sql().records(query, params, options.clone()).await? {
            println!("{row:?}");
        }
    }
    Ok(())
}
