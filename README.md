# scrib

Smooth freehand drawing in Rust + WASM.

```bash
wasm-pack build --target web --features wasm && cp pkg/* www/pkg/
cargo run -p scrib-server
```

| Command | |
|---|---|
| `cargo test` | Test |
| `cargo clippy` | Lint |
| `cargo fmt` | Format |
| `cargo run -p scrib-server` | Run server |
