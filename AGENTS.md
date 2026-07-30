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

## WASM Migration Tasks

### Done
1. Hit testing + selection/transform — move selection logic from JS to Rust
2. All rendering — Canvas2D calls from Rust via wasm-bindgen
3. State management — undo/redo in Rust
4. WebSocket + networking — in Rust via wasm-bindgen
5. Constants & pure utilities — `EPSILON`, `SEGMENTS`, `GRID`, `THROTTLE_*`, `CAP_FLOATS`, `TOOLS`, `screenToCanvas`, `distToSegment`, `snap`
6. Grid rendering — `drawGrid` in render.rs via web-sys
7. Animation loop — `tickAnimation` cursor/shape lerp math in Rust
8. Stroke finalization — `finalizeStroke` object construction in Rust
9. Message handlers — `onWSInit`, `onWSJoin`, `onWSLeave`, `onWSStroke*`, `onWSErase`, `onWSCursor`, `onWSShapeUpdate` in Rust
10. Transform logic — `selectHandleMove/Up` delta computation + `regenerate_mesh` in Rust

## Features
- `wasm` — enables wasm-bindgen bindings (default off)

## Commands
- Build WASM: `make build`
- Serve (HTTP): `make serve`
- Server (WS): `make server`
- Build + all servers: `make all`
- Test: `make test` / `cargo test`
- Lint: `make lint` / `cargo clippy`
- Format: `make fmt` / `cargo fmt`
