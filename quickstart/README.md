# quickstart

Connects with the SDK, creates `examples-rust/kb` (a text field, a keyword field and a 3-d cosine vector), upserts three documents and runs a hybrid search: kNN on the vector plus BM25 on `body`, fused with RRF and filtered to `tenant = "a"`.

```sh
LOAMS_ENDPOINT=http://127.0.0.1:8080 cargo run -p quickstart
```

Expected output:

```
connected to Loams 0.1.2 (http://127.0.0.1:8080)
created examples-rust/kb
upserted 3 documents
0.0328  refund policy
0.0161  shipping times
```

It drops and recreates `kb` on every run, so it is safe to repeat.
