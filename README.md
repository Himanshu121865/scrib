# scrib

Smooth freehand drawing in Rust + WASM.

## Features

- Variable-width brush strokes (velocity + pressure)
- Shapes: rectangle, circle, line, arrow
- Infinite canvas — scroll zoom, middle-drag pan
- Dark/light theme, grid snap, undo/redo
- Real-time multiplayer via WebSocket
- Zero-dependency Rust core; WASM in the browser

## Local Usage

```bash
# 1. Build WASM
wasm-pack build --target web --features wasm

# 2. Copy to web demo
cp pkg/* www/pkg/

# 3. Start the multiplayer server
cargo run -p scrib-server

# 4. Serve the web demo (new terminal)
cd www && python3 -m http.server 8080
```

Open `http://localhost:8080` in a browser. Append `#roomname` to the URL for multiplayer rooms.

## Commands

| Command | Description |
|---------|-------------|
| `cargo test` | Run tests |
| `cargo clippy` | Lint |
| `cargo fmt` | Format |
| `cargo run -p scrib-server` | Start server on `:9876` |
| `wasm-pack build --target web --features wasm` | Rebuild WASM |

## License

MIT
