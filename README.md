# scrib

Smooth freehand drawing in Rust + WASM.

## Quick start

```bash
make build      # build WASM + copy to www/pkg/
make all        # build + start the server (serves frontend + WebSocket)
make serve      # build + run server
make server     # run server (no build)
```

Open **http://localhost:9876** in a browser. Everyone connected to the server shares one canvas.

## Deploy to Render

1. Push the repo to GitHub.
2. On Render: **New → Web Service**, connect the repo, use `render.yaml` (or pick Docker runtime with the included `Dockerfile`).
3. Render sets `PORT` automatically; the server binds to it and serves both the frontend and the WebSocket on the same port.

Notes: free-tier services sleep after 15 min idle, and the canvas lives in memory — a restart clears it.

## Commands

| Command | Description |
|---------|-------------|
| `make build` | Build WASM + copy to `www/pkg/` |
| `make serve` | Build + run server on `:9876` (frontend + WebSocket) |
| `make server` | Run server on `:9876` (no build) |
| `make all` | Build + run server |
| `make test` | Run tests |
| `make lint` | Clippy |
| `make fmt` | Format |

## License

MIT
