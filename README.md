# scrib

Smooth freehand drawing library in Rust — like Excalidraw. Compiles to WASM for the web, with a multiplayer WebSocket server.

## Features

- **Variable-width strokes** — RDP simplification → centripetal Catmull-Rom smoothing → velocity+pressure width → offset curve mesh
- **Shape tools** — rect, circle, line, arrow (all rendered as Rust-generated triangle meshes)
- **Infinite canvas** — zoom toward cursor (scroll + 25% snap buttons), pan (middle-click / space+drag / two-finger pinch)
- **Dark/light theme** — orange accent (#e86a20), toggling switches active color
- **Grid overlay** — 30-unit snap-to-grid
- **Undo/redo** — 100-level stack
- **Multiplayer** — real-time cursor sync + stroke streaming via WebSocket

## Library

```rust
use scrib::{Point, Stroke};

let mut s = Stroke::new();
s.add_point(Point::new(0.0, 0.0, 0.5));
s.add_point(Point::new(1.0, 2.0, 0.3));
let out: Vec<[f64; 3]> = s.process_with_widths(2.5);
```

## Web Demo

```bash
# Build WASM module
wasm-pack build --target web --features wasm
cp pkg/* www/pkg/

# Run (serves both frontend + WebSocket)
cargo run -p scrib-server

# Open http://localhost:9876
```

The server listens on `$PORT` (default 9876) and handles both static files and WebSocket at `/ws`.

### Dev (separate processes)

```bash
# Terminal 1: WebSocket server
cargo run -p scrib-server

# Terminal 2: static file server (if not using axum server for files)
cd www && python3 -m http.server 8080
```

## Multiplayer Server

Standalone WebSocket server at `server/`. Manages rooms with real-time cursor sync, live stroke streaming, and persistent stroke history for late joiners.

```bash
cargo run -p scrib-server
```

Connects on `ws://host:port/ws`.

## Railway Deploy

```bash
wasm-pack build --target web --features wasm
cp pkg/* www/pkg/
git add www/pkg/
git commit -m "wasm build"
git push
```

Connect the repo to Railway — the Dockerfile handles the rest. The server auto-detects `$PORT`.

## Commands

| Command | Description |
|---|---|
| `cargo test` | Run all tests |
| `cargo clippy` | Lint |
| `cargo fmt` | Format |
| `cargo run -p scrib-server` | Run multiplayer server |

## Modules

| Module | Description |
|---|---|
| `point` | `Point { x, y, pressure }` |
| `stroke` | Stroke input + pipeline + width computation |
| `simplify` | Ramer-Douglas-Peucker simplification |
| `smooth` | Centripetal Catmull-Rom spline |
| `geometry` | Offset curves, mesh triangulation, shape generators, hit testing |
| `wasm` | wasm-bindgen bindings (`process_stroke`, `mesh_from_centerline`, `shape_mesh`, `hit_path`) |
