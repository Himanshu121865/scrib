# scrib

Smooth freehand drawing library (like Excalidraw). Pure Rust, compiles to WASM.

## Usage

### Native Rust

```rust
use scrib::{Point, Stroke};

let mut stroke = Stroke::new();
stroke.add_point(Point::new(0.0, 0.0, 0.5));
stroke.add_point(Point::new(1.0, 2.0, 0.3));
stroke.add_point(Point::new(3.0, 5.0, 0.7));
// ... more points

let smooth = stroke.process();
```

### Web (WASM)

```bash
wasm-pack build --target web --features wasm --out-dir www/pkg
cd www && python3 -m http.server 8080
```

Open `http://localhost:8080` — draw on the canvas.

### Run the demo

```bash
cargo run --example demo > demo.svg && xdg-open demo.svg
```

## Pipeline

```
raw points → RDP simplify → Catmull-Rom smooth → smooth path
```
