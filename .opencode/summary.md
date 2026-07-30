# Summary

## Objective
Build a smooth freehand drawing library (like Excalidraw) in Rust with variable-width strokes compiled to WASM, plus a multiplayer web demo.

## Important Details
- Pure Rust library, zero mandatory dependencies; `wasm-bindgen` behind a `wasm` feature
- Pipeline: raw points → RDP simplify → centripetal Catmull-Rom smooth → velocity+pressure width → output `[x, y, width, ...]`
- Stroke width: `base_size * (0.15 + 0.85 * p * (1.0 - 0.75 * speed))` with end tapering
- Incremental stroke processing: caches centerline + mesh, only regenerates mesh for the tail portion each move; throttled at 30ms; cache reset on stroke start/end/undo/redo/clear/tool-switch
- Cursor overlay canvas (`#cursorCanvas`) with `pointer-events: none`, `z-index: 5`; cursor updates only redraw cursor canvas via `redrawCursors()`, not the full scene
- `generate_mesh` refactored with internal `generate_mesh_inner(points, cap, closed)`; `generate_mesh_closed` public wrapper added
- Shape mesh generators: `rect_mesh`, `circle_mesh`, `line_mesh`, `arrow_mesh` — all produce triangle meshes in Rust
- Hit testing: `hit_path(px, py, data, width)` in Rust, replaces JS per-segment loop for path strokes
- Shapes rendered via Rust mesh (`drawShape` checks `.mesh` property first, canvas stroke fallback for legacy data); remote shapes get mesh generated on receipt
- All strokes assigned unique `id` at creation; eraser sends IDs via `erase` WS message; server removes from storage + broadcasts; `onWSErase` handler removes by ID + `saveState`
- Remote cursor smoothing via lerp: `rx,ry` render positions approach `tx,ty` targets at 0.25/frame in `tickAnimation()` rAF loop, snaps at 0.5px threshold
- Remote live shape smoothing via lerp: `onWSShapeUpdate` sets `tx1,ty1,tx2,ty2` targets; `tickAnimation()` lerps `rx1,ry1,rx2,ry2` and regenerates mesh each frame; first update snaps render to target with immediate `redraw()`
- Eraser button uses SVG icon instead of `◇`
- Keybinds: `D` draw, `E` erase, `R` rect, `C` circle, `L` line, `A` arrow, `G` grid, `Esc` cancel, `Ctrl+Z` undo, `Ctrl+Shift+Z`/`Ctrl+Y` redo
- Infinite canvas: zoom toward cursor (scroll + 25% snap buttons), pan (middle-click / space+drag / two-finger pinch)
- Dark + orange theme (#e86a20 accent); `.tog-on` has orange background fill; bgToggle swaps moon/sun SVG icon but doesn't get highlight; toggling dark mode switches active color between #eee and #222
- Grid overlay (30-unit, togglable, adapts to background); shapes snap to grid when on
- Undo/redo: 100-level stack, state saved before every stroke/erase/clear
- Server (`server/`): Rust tokio-tungstenite WebSocket server on `ws://0.0.0.0:9876`; room management, stroke storage for late joiners, cursor/stroke/shape streaming via WS
- Server ensures every stored stroke has `id`: `stroke-end` handler injects server-generated `srv_*` ID if missing; join handler migrates all legacy stored strokes with `srv_user_idx` IDs
- Dot strokes now have `id: currentStrokeId` in `finalizeStroke`
- Erase cross-client fix: server broadcasts erased stroke owners (`owners: Vec<UserId>`) alongside IDs; local strokes set `userId: myId` at creation; `onWSErase` falls back to `userId` matching when `id` doesn't match — covers old strokes (missing `id`), new strokes (`id` match), and remote strokes (`userId` match)
- Railway deploy plan: axum server serving both static files + WebSocket on same port; pre-build WASM and commit `www/pkg/` to git; Dockerfile at project root
- AGENTS.md untracked from git and not in .gitignore (user preference)
- Repo: `https://github.com/Himanshu121865/scrib`

## Work State
### Completed
- Core pipeline: all processing modules with tests (22 passing across `geometry`, `simplify`, `smooth`, `stroke`)
- Rust geometry engine (`src/geometry.rs`): offset curves, miter joints, round caps, closed-path support, triangulated mesh output → flat `[x, y, ...]` triangle vertex array; shape mesh generators: `rect_mesh`, `circle_mesh`, `line_mesh`, `arrow_mesh`; hit testing: `hit_path`
- WASM bindings: `process_stroke()`, `mesh_from_centerline()`, `shape_mesh(kind, x1, y1, x2, y2, width, segments)`, `hit_path(px, py, data, width)`
- Web demo: full Excalidraw-like UI with left tool panel (all tools use SVG icons), top bar (undo/redo/clear/zoom/grid toggle with SVG/bg toggle with moon-sun SVG/help), bottom bar (color swatches + custom color picker + size slider)
- Default brush size 10 (range 1–20); shapes share same size variable
- All rendering uses Rust-generated triangle mesh (`drawMesh` single `fill()` call); shapes use `shape_mesh` when `.mesh` available with `drawShape` canvas fallback
- Incremental stroke processing (`processStrokeIncremental`): centerline + mesh caching, tail-only mesh regeneration, 30ms throttle, cache reset on state changes
- Cursor overlay canvas (`cursorCanvas`, `cursorCtx`): cursor updates call `redrawCursors()` instead of full `redraw()`
- Multiplayer erase: unique `id` on every stroke, `erase` WS message type, server removes from storage + broadcasts, `onWSErase` handler with `saveState`
- Eraser button: SVG icon replacing `◇`
- `.tog-on` active state: orange background fill; bgToggle excluded from highlight
- Grid: `G` keybind added, shortcut panel entry
- README updated: minimal format with Railway deploy steps
- Multiplayer server (`server/`): Rust WebSocket server using tokio-tungstenite on `ws://0.0.0.0:9876`; room management, stroke storage/cursor sync/stroke streaming
- Remote cursor smoothing: `rx,ry` lerp toward `tx,ty` via `tickAnimation()` rAF loop at 0.25/frame with 0.5px snap threshold
- Remote live shape smoothing: `onWSShapeUpdate` stores target positions, `tickAnimation()` lerps render positions and regenerates mesh each frame, first update snaps with immediate `redraw()`
- Dot strokes: added `id: currentStrokeId` in `finalizeStroke`
- Server: ensures `id` field on all stored strokes — `stroke-end` handler injects `srv_*` ID if missing; join handler migrates legacy strokes with `srv_user_idx` IDs
- Erase cross-client fix: server broadcasts `owners` (creator `userId`) alongside `ids`; local strokes get `userId: myId` at creation; `onWSErase` falls back to `userId` match when `id` doesn't match — covers old strokes (no `id`), new strokes (`id` match), and remote strokes (`userId` match)

### Active
- (none)

### Blocked
- (none)

## Next Move
- Railway deploy: build Dockerfile, verify `www/pkg/` committed, push to Railway

## Relevant Files
- `src/lib.rs` — module declarations, re-exports `Point`, `Stroke`, `generate_mesh`
- `src/geometry.rs` — offset curve generation, cap arcs, closed-path support, shape mesh generators (`rect_mesh`, `circle_mesh`, `line_mesh`, `arrow_mesh`), `hit_path` function
- `src/point.rs` — `Point { x, y, pressure }`
- `src/simplify.rs` — `rdp()` function
- `src/smooth.rs` — `catmull_rom()` with centripetal alpha=0.5
- `src/stroke.rs` — `Stroke` struct, `pipeline()`, `compute_widths()`
- `src/wasm.rs` — `process_stroke()`, `mesh_from_centerline()`, `shape_mesh(kind, x1, y1, x2, y2, width, segments)`, `hit_path(px, py, data, width)` WASM exports
- `www/index.html` — full web demo with all tools, incremental stroke processing, cursor overlay canvas, multiplayer client with erase sync, live stroke/shape streaming, remote cursor + shape smoothing via `tickAnimation()`, erase with `userId` fallback matching, local strokes set `userId: myId`
- `server/Cargo.toml` — tokio, tokio-tungstenite, serde/serde_json, futures-util
- `server/src/main.rs` — WebSocket server with room management, stroke storage, cursor/join/leave/erase broadcast, ID migration for legacy strokes, erase broadcast includes `owners` (creator userId)
- `Cargo.toml` — workspace root, `wasm-bindgen` optional dep, `wasm` feature, `cdylib` crate-type
