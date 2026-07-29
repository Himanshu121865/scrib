# scrib

Smooth freehand drawing in Rust + WASM. Like Excalidraw.

```bash
wasm-pack build --target web --features wasm
cp pkg/* www/pkg/
cargo run -p scrib-server
```

```rust
use scrib::{Point, Stroke};
let mut s = Stroke::new();
s.add_point(Point::new(0.0, 0.0, 0.5));
s.add_point(Point::new(1.0, 2.0, 0.3));
let out: Vec<[f64; 3]> = s.process_with_widths(2.5);
```

## Commands

| `cargo test` | Test |
| `cargo clippy` | Lint |
| `cargo fmt` | Format |
| `cargo run -p scrib-server` | Run multiplayer server |

## Railway

```bash
wasm-pack build --target web --features wasm && cp pkg/* www/pkg/
git add www/pkg/ && git commit -m "wasm build" && git push
```

Connect repo to Railway — Dockerfile handles the rest.
