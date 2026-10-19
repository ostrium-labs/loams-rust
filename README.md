# Loams Rust examples

[Loams](https://loams.dev) is one engine for hybrid retrieval: vector search, BM25 full text and filters fused in a single query, over documents on object storage. This repository holds example apps for the Rust SDK (`loams` on crates.io).

| Example | What it shows |
|---|---|
| [`quickstart`](quickstart) | Connect, create a collection, upsert documents, hybrid search (vector + BM25 + filter) |
| [`sql`](sql) | SQL over a collection: `SELECT`, and the `rrf`, `vector_search` and `text_search` table functions |
| [`search-service`](search-service) | An axum HTTP service that answers `GET /search` from a collection |

## Requirements

- Rust 1.97 or newer (`rust-toolchain.toml` pins it) and `protoc` (`apt install protobuf-compiler`, `brew install protobuf`), because the generated protobuf code is built from the `loams-proto` crate.
- A running Loams engine. Either run one from source (`cargo run -p loams -- dev` in the [Loams repository](https://github.com/ostrium-labs/loams)) or start the compose stack from [docs/release/preview.md](https://github.com/ostrium-labs/loams/blob/main/docs/release/preview.md).

## Run

The examples read `LOAMS_ENDPOINT` (default `http://127.0.0.1:8080`, which is what `loams dev` serves) and write only to the `examples-rust` namespace.

```sh
export LOAMS_ENDPOINT=http://127.0.0.1:8080
cargo run -p quickstart        # creates and fills examples-rust/kb
cargo run -p sql
cargo run -p search-service    # then: curl 'localhost:3737/search?q=refund&tenant=a'
```

Run `quickstart` first: the other two read the collection it creates. These examples assume an engine with authentication off, as `loams dev` is. For an instance with authentication, add `.with_auth(Arc::new(ApiKey::new(key)?))` to the `Loams` client and pass the same credential to the other calls.

## About the SDK version

The `loams` crate's 0.1.0 release is not on crates.io yet, so these examples pin `0.1.0-preview.2`. The SDK in the monorepo (`sdks/rust`) is the same client. In this version the SDK covers the instance, system and live modules; the collection calls in `quickstart` and `search-service` go through the generated `loams-proto` clients over the SDK's own transport, and SQL is a JSON `POST` to the native API. When 0.1.0 is published, bump the version in each `Cargo.toml`.

## Links

- Docs: <https://loams.dev/docs>
- Running Loams: [docs/release/preview.md](https://github.com/ostrium-labs/loams/blob/main/docs/release/preview.md)
- SDK source: <https://github.com/ostrium-labs/loams/tree/main/sdks/rust>

Licensed under Apache-2.0.
