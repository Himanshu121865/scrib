# scrib

Smooth freehand drawing library (like Excalidraw). Pure Rust, compiles to WASM.

## Features

- **Freehand brush** — variable-width strokes driven by velocity and pressure
- **Shapes** — rectangle, circle, line, arrow
- **Smoothing** — centripetal Catmull-Rom spline interpolation
- **Simplification** — Ramer-Douglas-Peucker point reduction
- **Eraser** — drag-to-erase with preview
- **Unlimited undo/redo** — 100-level history stack
- **Infinite canvas** — pan (middle-click / space+drag), zoom (scroll wheel toward cursor, 25% snap buttons)
- **Grid overlay** — togglable, adapts to background
- **Dark/light mode** — toggles background and auto-switches pen color
- **Pressure-aware** — from pen/tablet input

## Library (Rust)

```rust
use scrib::{Point, Stroke};

let mut stroke = Stroke::new();
stroke.add_point(Point::new(0.0, 0.0, 0.5));
stroke.add_point(Point::new(1.0, 2.0, 0.3));
stroke.add_point(Point::new(3.0, 5.0, 0.7));

// Smooth centerline
let smooth: Vec<Point> = stroke.process();

// Variable-width geometry
let output: Vec<[f64; 3]> = stroke.process_with_widths(2.5);
// each entry is [x, y, width]

// Or use the free functions directly:
use scrib::stroke::{pipeline, compute_widths};
let pts = pipeline(&raw_points, 0.5, 6);
let widths = compute_widths(&pts, 2.5);
```

## Web Demo

```bash
wasm-pack build --target web --features wasm
cp -r pkg/* www/pkg/
cd www && python3 -m http.server 8080
```

Open `http://localhost:8080`.

### Run the CLI demo

```bash
cargo run --example demo > demo.svg
```

## Pipeline

```
raw points → RDP simplify → Catmull-Rom smooth → compute_widths → [x, y, width] output
```
