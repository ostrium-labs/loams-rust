# sql

Runs SQL over `examples-rust/kb` through the published SDK's `loams.sql` module: a plain `SELECT`, a parameterized aggregate, and `rrf(vector_search(...), text_search(...))`. Run `quickstart` first.

```sh
LOAMS_ENDPOINT=http://127.0.0.1:8080 cargo run -p sql
```
