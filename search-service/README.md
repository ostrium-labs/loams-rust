# search-service

A small [axum](https://docs.rs/axum) service in front of Loams. `GET /search` builds a hybrid query with the SDK's `Search` and `VectorOptions` builders and calls the typed `loams.search` module. It does not build raw RPC messages. Run `quickstart` first.

```sh
LOAMS_ENDPOINT=http://127.0.0.1:8080 cargo run -p search-service
curl 'localhost:3737/search?q=refund&v=1,0,0&tenant=a'
```

Parameters: `q` (required, the text), `v` (a comma-separated 3-d query vector, default `1,0,0`), `tenant` (optional filter), `limit` (default 5). The service listens on `127.0.0.1:3737`; set `LISTEN` to change it.
