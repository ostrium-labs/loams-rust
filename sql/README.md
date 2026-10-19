# sql

Runs SQL over `examples-rust/kb` through the native API's `/v1/namespaces/{ns}/sql` route: a plain `SELECT`, then `rrf(vector_search(...), text_search(...))`, which is the hybrid query from `quickstart` written in SQL. Run `quickstart` first.

```sh
LOAMS_ENDPOINT=http://127.0.0.1:8080 cargo run -p sql
```
