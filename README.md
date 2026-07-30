# scrib

Smooth freehand drawing in Rust + WASM.

## Quick start

```bash
make build      # build WASM + copy to www/pkg/
make all        # build + start both servers (WebSocket + HTTP)
make serve      # HTTP server only (terminal 1)
make server     # WebSocket server only (terminal 2)
```

Open **http://localhost:8080** in a browser. Append `#roomname` to the URL for multiplayer.

## Commands

| Command | Description |
|---------|-------------|
| `make build` | Build WASM + copy to `www/pkg/` |
| `make serve` | HTTP server on `:8080` |
| `make server` | WebSocket server on `:9876` |
| `make all` | Build + start both servers |
| `make test` | Run tests |
| `make lint` | Clippy |
| `make fmt` | Format |

## License

MIT
