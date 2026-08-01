# scrib

Smooth freehand drawing in the browser, written in Rust.

Try it live: **https://scrib-izfp.onrender.com**

## Install

```bash
cargo install wasm-pack
git clone https://github.com/Himanshu121865/scrib
cd scrib
```

## Usage

```bash
make build      # build WASM + copy to www/pkg/
make all        # build + run server (frontend + WebSocket on :9876)
make serve      # build + run server
make server     # run server (no build)
```

Open **http://localhost:9876** — everyone connected to the server shares one canvas.

## Features

- Pressure-sensitive strokes with velocity-based width
- Shape tools: rectangle, circle, line, arrow
- Rubber-band multi-select with move and delete
- Real-time multi-user cursors and shared canvas
- Brush controls: size, smoothing, simplification, velocity
- Collapsible toolbar and color palette

## License

MIT
