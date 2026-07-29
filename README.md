# scrib

Smooth freehand drawing library in Rust, compiled to WASM. Multiplayer web demo included.

Algorithm pipeline: raw points → RDP simplify → Catmull-Rom smooth → velocity/pressure-based variable width → offset curves → triangulated mesh.

## Build & Run

```bash
# Build WASM + copy to web demo
wasm-pack build --target web --features wasm && cp pkg/* www/pkg/

# Start multiplayer server
cargo run -p scrib-server
```

Then open `www/index.html` in a browser (or serve it — `python3 -m http.server 8080`).

## Commands

| Command | |
|---|---|
| `cargo test` | Run tests |
| `cargo clippy` | Lint |
| `cargo fmt` | Format |
| `cargo run -p scrib-server` | Start WebSocket server on `:9876` |

## Architecture

| Layer | |
|---|---|
| `src/` | Core Rust library — geometry, simplification, smoothing, stroke pipeline |
| `src/wasm.rs` | WASM bindings via `wasm-bindgen` |
| `server/` | Multiplayer WebSocket server (tokio-tungstenite) |
| `www/` | Browser client — vanilla JS, Canvas2D rendering |
