# Loams Rust examples

These examples target the published **Loams SDK 0.1.2** release. Cargo resolves
the Rust SDK crates from crates.io.

[Loams](https://loams.dev) is one engine for hybrid retrieval: vector search, BM25 full text and filters fused in a single query, over documents on object storage. This repository holds example apps for the Rust SDK (`loams` on crates.io).

| Example | What it shows |
|---|---|
| [`quickstart`](quickstart) | Instance/system, namespaces, collections, documents, text/vector/hybrid search, optional graph, SDK SQL and streams |
| [`sql`](sql) | SDK SQL module: `SELECT`, bound parameters, and the `rrf`, `vector_search` and `text_search` table functions |
| [`search-service`](search-service) | An axum HTTP service whose handler uses the SDK search builder/module |

## Requirements

- Rust 1.97 or newer (`rust-toolchain.toml` pins it) and `protoc` (`apt install protobuf-compiler`, `brew install protobuf`), because the generated protobuf code is built from the `loams-proto` crate.
- The SDK's default TLS feature is enabled, so `LOAMS_ENDPOINT` can use either `http://` or `https://`.
- A running Loams engine. Either run one from source (`cargo run -p loams -- dev` in the [Loams repository](https://github.com/ostrium-labs/loams)) or start the compose stack from [docs/release/preview.md](https://github.com/ostrium-labs/loams/blob/main/docs/release/preview.md).

## Run

The SDK requires `LOAMS_ENDPOINT`; set it to `http://127.0.0.1:8080` for `loams dev`. The examples write only to the `examples-rust` namespace.

```sh
export LOAMS_ENDPOINT=http://127.0.0.1:8080
cargo run -p quickstart        # creates and fills examples-rust/kb
cargo run -p sql
cargo run -p search-service    # then: curl 'localhost:3737/search?q=refund&tenant=a'
```

Run `quickstart` first: the other two read the collection it creates. These examples assume an engine with authentication off, as `loams dev` is. For an instance with authentication, add `.with_auth(Arc::new(ApiKey::new(key)?))` to the `Loams` client and pass the same credential to the other calls.

## About the SDK version

The examples pin `loams` to `=0.1.2` and use the SDK's typed modules and builders;
they do not construct raw Connect clients or call the SQL HTTP route directly.
The SDK's `live()` module remains available for Live sync, while `system()` is
demonstrated in `quickstart`. See the [SDK notes example](https://github.com/ostrium-labs/loams/blob/v0.1.2/sdks/rust/examples/notes/main.rs)
for the same published module signatures.

The SDK crates are pinned to `=0.1.2`. CI generates a workspace lockfile from
the published crates, then builds with `--locked`.

## Links

- Docs: <https://loams.dev/docs>
- Running Loams: [docs/release/preview.md](https://github.com/ostrium-labs/loams/blob/main/docs/release/preview.md)
- SDK source: <https://github.com/ostrium-labs/loams/tree/main/sdks/rust>

Licensed under Apache-2.0.
