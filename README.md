# scrib

Smooth freehand drawing in Rust + WASM. Like Excalidraw, but your own.

## Features

- Variable-width brush strokes driven by velocity and pressure
- Shapes: rectangle, circle, line, arrow
- Eraser with hit testing
- Infinite canvas — scroll to zoom, middle-click or Space+drag to pan
- Dark/light theme, grid snap, undo/redo (100 levels)
- **Multiplayer** — real-time sync via WebSocket, remote cursor smoothing, live stroke preview
- Zero dependency Rust core; WASM in the browser via `wasm-bindgen`

## Quick Start

```bash
# Build the WASM module
wasm-pack build --target web --features wasm
cp pkg/* www/pkg/

# Start the multiplayer server (from project root)
cargo run -p scrib-server

# Serve the web demo
cd www && python3 -m http.server 8080
```

Open `http://localhost:8080` in a browser. Append `#roomname` to the URL to create/join a named room — anyone on the same room sees your strokes in real time.

## Usage

### Tools

| Key | Tool |
|-----|------|
| `D` | Pen — draw freehand strokes |
| `E` | Eraser — click or drag through strokes |
| `R` | Rectangle |
| `C` | Circle |
| `L` | Line |
| `A` | Arrow |
| `G` | Toggle grid |
| `Esc` | Cancel current tool |

### Canvas

| Action | |
|--------|------|
| Scroll | Zoom toward cursor |
| `+` / `−` buttons | Zoom in 25% steps |
| `Fit` button | Reset zoom to 100% |
| Middle-click drag | Pan |
| Space + left-click drag | Pan |
| Two-finger pinch (touch) | Zoom + pan |

### Undo/Redo

| Key | Action |
|-----|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` or `Ctrl+Y` | Redo |

## Library API (Rust)

```rust
use scrib::{Point, Stroke};

let mut s = Stroke::new();
s.add_point(Point::new(0.0, 0.0, 0.5));
s.add_point(Point::new(1.0, 2.0, 0.3));
let out: Vec<[f64; 3]> = s.process_with_widths(2.5);
```

The output is `[x, y, width]` triples ready for mesh generation.

### Pipeline

```
raw points → RDP simplify → Catmull-Rom smooth → width computation → output
```

Centripetal Catmull-Rom interpolation (`alpha = 0.5`) produces smooth curves through all input points. Width is computed per-segment from velocity and pressure:

```
width = base_size × (0.15 + 0.85 × pressure × (1.0 − 0.75 × speed))
```

### WASM Exports

| Function | Returns | Description |
|----------|---------|-------------|
| `process_stroke(raw, epsilon, segments, size)` | `Float64Array` | Full pipeline: raw coords → `[x, y, width, ...]` |
| `mesh_from_centerline(data)` | `Float64Array` | Offset curves → triangulated `[x, y, ...]` |
| `shape_mesh(kind, x1, y1, x2, y2, width, segs)` | `Float64Array` | Shape triangle mesh (rect/circle/line/arrow) |
| `hit_path(px, py, data, width)` | `bool` | Point-in-stroke hit test |

## Architecture

```
┌──────────────────────────────────────────────────┐
│  www/  (vanilla JS + Canvas2D)                   │
│  ├── app.js       entry, UI, pointer events      │
│  ├── state.js     shared mutable state           │
│  ├── render.js    redraw, grid, cursors          │
│  ├── tools.js     draw/erase/shape handlers      │
│  ├── network.js   WebSocket message handlers     │
│  ├── style.css                                   │
│  └── index.html   skeleton                       │
├──────────────────────────────────────────────────┤
│  pkg/   (compiled WASM — scrib.d.ts, scrib.js)   │
├──────────────────────────────────────────────────┤
│  src/   (Rust library, zero deps)                │
│  ├── point.rs       Point { x, y, pressure }     │
│  ├── simplify.rs    Ramer-Douglas-Peucker        │
│  ├── smooth.rs      Catmull-Rom interpolation    │
│  ├── stroke.rs      pipeline + width computation │
│  ├── geometry.rs    offset curves, mesh, shapes  │
│  └── wasm.rs        wasm-bindgen bindings        │
├──────────────────────────────────────────────────┤
│  server/ (Rust, tokio-tungstenite)               │
│  └── main.rs        rooms, strokes, broadcast    │
└──────────────────────────────────────────────────┘
```

## Development

| Command | |
|---------|---|
| `cargo test` | Run all unit tests |
| `cargo clippy` | Lint |
| `cargo fmt` | Format |
| `cargo run -p scrib-server` | Start WebSocket server on `:9876` |
| `wasm-pack build --target web --features wasm` | Rebuild WASM |
| `python3 -m http.server 8080` | Serve web demo (from `www/`) |

### Prerequisites

- Rust toolchain (rustup, cargo)
- `wasm-pack` — `cargo install wasm-pack` or via your package manager
- Python 3 (for the dev HTTP server)

## License

MIT
