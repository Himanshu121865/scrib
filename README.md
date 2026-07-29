# scrib

Smooth freehand drawing (like Excalidraw). Pure Rust, compiles to WASM.

```bash
wasm-pack build --target web --features wasm
cp -r pkg/* www/pkg/
cd www && python3 -m http.server 8080
```

```rust
use scrib::{Point, Stroke};
let mut s = Stroke::new();
s.add_point(Point::new(0.0, 0.0, 0.5));
s.add_point(Point::new(1.0, 2.0, 0.3));
let out: Vec<[f64; 3]> = s.process_with_widths(2.5);
```
