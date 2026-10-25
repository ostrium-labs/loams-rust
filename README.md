# Loams Rust examples

These examples target **Loams SDK 0.1.2**. Installation and CI require the matching
0.1.2 packages and release tags to be published.

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

The examples pin `loams` (and `loams-proto` where used) to `=0.1.2`.
The SDK provides collection, search, SQL and stream modules as well as instance,
system and Live. These examples retain generated collection clients over the SDK
transport and the native SQL HTTP route. See the [SDK notes example](https://github.com/ostrium-labs/loams/blob/v0.1.2/sdks/rust/examples/notes/main.rs)
for builders, collection handles and SDK SQL.

After publication, run `cargo generate-lockfile` and commit `Cargo.lock`. CI
resolves the new release before building with `--locked`.

## Links

- Docs: <https://loams.dev/docs>
- Running Loams: [docs/release/preview.md](https://github.com/ostrium-labs/loams/blob/main/docs/release/preview.md)
- SDK source: <https://github.com/ostrium-labs/loams/tree/main/sdks/rust>

Licensed under Apache-2.0.
