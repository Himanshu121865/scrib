# AGENTS.md

## Project
`scrib` - Rust library for smooth freehand drawing (like Excalidraw).

## Goal
Build a smooth drawing algorithm — freehand brush strokes without rigid lines.

## Strategy
- Pure Rust library, zero dependencies (math from scratch)
- Algorithm pipeline: raw points → simplify (RDP) → smooth (Catmull-Rom) → variable width (velocity + pressure) → offset curves → round caps → triangulated geometry
- Pressure-aware from the start (pressure field on point)

## Modules
- `point` — `Point { x, y, pressure }`
- `stroke` — stroke input (raw points) + parameters
- `simplify` — Ramer-Douglas-Peucker simplification
- `smooth` — Catmull-Rom spline interpolation
- `wasm` — wasm-bindgen bindings for web

## Features
- `wasm` — enables wasm-bindgen bindings (default off)

## Commands
- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy`
- Format: `cargo fmt`
- WASM: `wasm-pack build --target web --features wasm --out-dir www/pkg`
- Serve: `python3 -m http.server 8080` in `www/`
