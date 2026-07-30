# scrib

> Smooth freehand drawing in Rust + WASM. Like Excalidraw, but your own.

![build](https://img.shields.io/badge/build-passing-brightgreen)
![rust](https://img.shields.io/badge/rust-1.70+-orange)
![wasm](https://img.shields.io/badge/wasm-ready-purple)
![license](https://img.shields.io/badge/license-MIT-blue)
![size](https://img.shields.io/badge/size-zero%20deps-success)

## Features

- Variable-width brush strokes driven by velocity and pressure
- Shapes: rectangle, circle, line, arrow
- Eraser with hit testing
- Infinite canvas — scroll to zoom, middle-click or Space+drag to pan
- Dark/light theme, grid snap, undo/redo (100 levels)
- **Multiplayer** — real-time sync via WebSocket, remote cursor smoothing, live stroke preview
- Zero dependency Rust core; WASM in the browser via `wasm-bindgen`

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
  - [Multiplayer Rooms](#multiplayer-rooms)
- [Usage](#usage)
  - [Tools](#tools)
  - [Canvas Controls](#canvas)
  - [Undo/Redo](#undoredo)
  - [Color & Size](#color--size)
  - [Dark Mode](#dark-mode)
  - [Grid](#grid)
- [Algorithm](#algorithm)
  - [Stroke Pipeline](#stroke-pipeline)
  - [Width Formula](#width-formula)
  - [Shape Meshes](#shape-meshes)
  - [Hit Testing](#hit-testing)
- [Using scrib as a Rust Library](#using-scrib-as-a-rust-library)
  - [Core Types](#core-types)
  - [Processing Pipeline](#processing-pipeline-1)
  - [Step-by-Step API](#step-by-step)
  - [Mesh Generation](#mesh-generation-1)
  - [Pipeline Summary](#pipeline-summary)
  - [WASM Exports](#wasm-exports)
- [Algorithmic Details](#algorithmic-details)
  - [Ramer-Douglas-Peucker Simplification](#ramer-douglas-peucker-simplification)
  - [Centripetal Catmull-Rom Spline](#centripetal-catmull-rom-spline)
  - [Offset Curves](#offset-curves)
  - [Normal and Tangent Vectors](#normal-and-tangent-vectors)
  - [Miter Joints](#miter-joints)
  - [Capping](#capping)
  - [Triangulation](#triangulation)
- [Multiplayer](#multiplayer)
  - [Room System](#room-system)
  - [Sync Details](#what-syncs)
  - [Erase Ownership](#erase-ownership)
  - [WebSocket Protocol](#websocket-protocol)
- [Data Model](#data-model)
  - [Stroke Object](#stroke-object)
  - [Shape Object](#shape-object)
  - [Storage](#storage)
- [Server Architecture](#server-architecture)
  - [Key Design Decisions](#key-design-decisions)
- [Client State Management](#client-state-management)
  - [Design Decisions](#design-decisions)
  - [State Persistence](#state-persistence)
  - [State Reset Triggers](#state-reset-triggers)
- [Dependency Graph](#dependency-graph)
  - [Why Zero Rust Dependencies in Core?](#why-zero-rust-dependencies-in-core)
  - [Why Tokio in the Server?](#why-tokio-in-the-server)
- [Client Rendering](#client-rendering)
  - [Rendering Pipeline](#rendering-pipeline-1)
  - [drawMesh](#drawmesh)
  - [Cursor Overlay](#cursor-overlay)
  - [Animation Loop](#animation-loop)
- [Incremental Rendering](#incremental-rendering)
- [Performance](#performance)
  - [WASM Mesh Generation](#wasm-mesh-generation)
  - [Incremental Processing](#incremental-processing-1)
  - [Rendering Path](#rendering-path)
  - [Memory](#memory)
- [Testing](#testing)
  - [Rust Unit Tests](#rust-unit-tests)
  - [Test Categories](#test-categories)
  - [Running Tests](#running-tests)
- [Architecture](#architecture)
- [Configuration](#configuration)
- [Development](#development)
  - [Prerequisites](#prerequisites)
  - [Commands](#commands)
  - [Build Pipeline](#build-pipeline)
  - [Project Structure](#project-structure)
  - [Browser Client Modules](#browser-client-modules)
  - [UI Layout](#ui-layout)
- [Comparison](#comparison)
- [Troubleshooting](#troubleshooting)
- [FAQ](#faq)
- [Contributing](#contributing)
  - [Getting Started](#getting-started)
   - [Coding Standards](#coding-standards)
  - [Help Wanted](#what-needs-help)
- [Future Work](#future-work)
- [Changelog](#changelog)
- [Security](#security)
- [License](#license)

## Quick Start

```bash
# 1. Build the WASM module
wasm-pack build --target web --features wasm

# 2. Copy artifacts to web demo directory
cp pkg/* www/pkg/

# 3. Start the multiplayer server
cargo run -p scrib-server

# 4. Serve the web demo (leave the server running in another terminal)
cd www && python3 -m http.server 8080
```

Open `http://localhost:8080` in a browser and start drawing.

### Multiplayer Rooms

Append `#roomname` to the URL — anyone on the same hash sees your strokes in real time. Without a hash, a random 5-character room is assigned automatically.

```text
http://localhost:8080#sketch-night
http://localhost:8080#team-whiteboard
```

## Usage

### Tools

| Key | Tool | Description |
|-----|------|-------------|
| `D` | Pen | Freehand drawing with variable-width strokes |
| `E` | Eraser | Click or drag through strokes to erase (own strokes only) |
| `R` | Rectangle | Click-drag to place a rectangle |
| `C` | Circle | Click-drag to place a circle (diameter = distance) |
| `L` | Line | Click-drag to draw a straight line |
| `A` | Arrow | Click-drag to draw an arrow |
| `G` | Grid | Toggle grid overlay on/off |
| `Esc` | Cancel | Cancel current shape placement |

### Canvas Controls

| Input | Action |
|-------|--------|
| Scroll wheel | Zoom toward cursor position |
| `+` button | Zoom in 25% |
| `−` button | Zoom out 25% |
| `Fit` button | Reset zoom to 100%, center view |
| Middle mouse drag | Pan the canvas |
| Space + left drag | Pan the canvas |
| Two-finger pinch | Zoom + pan (touch devices) |

### Undo/Redo

| Key | Action |
|-----|--------|
| `Ctrl+Z` | Undo last action |
| `Ctrl+Shift+Z` or `Ctrl+Y` | Redo last undone action |

Undo/redo supports 100 levels. The stack captures both strokes and shapes.

### Color & Size

**Color swatches** (bottom bar, left): black, red, orange, yellow, green, cyan, blue, purple, magenta, white. Click to select. Active color gets a selection ring.

**Custom color**: Click the `+` button next to swatches to open the browser's native color picker.

**Brush size**: Drag the slider (1–20) to adjust the base width of strokes and shapes. The circular preview updates in real time to show the selected width.

### Dark Mode

Toggle dark mode with the moon/sun icon in the top bar. Dark mode switches the canvas background to `#222` and uses orange (`#e86a20`) accents for UI highlights. Light mode uses a `#eee` canvas.

### Grid

Press `G` or click the grid toggle button. The grid is drawn as 30-unit squares over the canvas. When grid snap is enabled, shape anchors snap to the nearest grid intersection during placement.

### Keyboard Shortcuts Summary

| Key | Action |
|-----|--------|
| `D` | Pen tool |
| `E` | Eraser tool |
| `R` | Rectangle tool |
| `C` | Circle tool |
| `L` | Line tool |
| `A` | Arrow tool |
| `G` | Toggle grid |
| `Esc` | Cancel shape / deselect |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl+Y` | Redo |
| `Space` (hold) | Temporary pan mode |

## Multiplayer

Each browser tab gets a unique ID from the server. Strokes, cursors, and shape previews are streamed in real time through a persistent WebSocket connection to port 9876.

### Room System

The room name is taken from the URL hash (`#roomname`). Tabs sharing the same hash are in the same room. If no hash is provided, a random 5-character room is generated server-side.

```text
http://localhost:8080#my-room   → room "my-room"
http://localhost:8080            → room "a3k9m" (random)
```

### What Syncs

| Data | Interval | Details |
|------|----------|---------|
| Strokes | On stroke end | Full stroke centerline + pre-built triangle mesh |
| Live strokes | Every 80ms | Incremental mesh updates while drawing in progress |
| Live shapes | Every 80ms | Position updates client-side interpolated via lerp |
| Cursors | Every 40ms | Lerp-smoothed at 25%/frame, snaps at 0.5px threshold |
| Erase | On erase | Broadcasts stroke IDs + creator user IDs |

### Erase Ownership

Users can only erase their own strokes. The server verifies `s.user_id == user_id` before removing a stroke from storage. On the client, local hit-testing skips strokes where `stroke.userId !== myId`.

### WebSocket Protocol

Messages are newline-delimited JSON (`\n`). Each message has a `type` field.

**Client → Server**

| Type | Payload | When |
|------|---------|------|
| `join` | `{ room }` | Page load |
| `stroke-start` | `{ id }` | Pen down |
| `stroke-update` | `{ id, data }` | Pen move (throttled) |
| `stroke-end` | `{ id, data, color, mesh, userId }` | Pen up |
| `shape-update` | `{ id, kind, x1, y1, x2, y2, color, width }` | Shape drag |
| `cursor` | `{ x, y }` | Pointer move (throttled) |
| `erase` | `{ ids, owners }` | After erasing strokes |

**Server → Client**

| Type | Payload | When |
|------|---------|------|
| `init` | `{ userId, room }` | On connect |
| `join` | `{ userId }` | Peer joined |
| `leave` | `{ userId }` | Peer left |
| `stroke-end` | `{ id, data, color, mesh, userId, mesh }` | Peer finished a stroke |
| `stroke-update` | `{ id, data, userId }` | Peer is drawing |
| `shape-update` | `{ id, kind, x1, y1, x2, y2, color, width, userId }` | Peer is placing a shape |
| `erase` | `{ ids, owners }` | Peer erased strokes |
| `cursor` | `{ userId, x, y }` | Peer moved cursor |

## Algorithm

The drawing engine transforms raw pointer input into smooth, variable-width triangle meshes through a multi-stage pipeline.

### Stroke Pipeline

```text
                    ┌─────────────┐
  raw mouse events  │  collect    │
  ─────────────────►│  points     │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  RDP        │  Ramer-Douglas-Peucker
                    │  simplify   │  epsilon = 0.3
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  Catmull-   │  centripetal alpha = 0.5
                    │  Rom smooth │  inserts 8 segments per pair
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  width      │  velocity + pressure formula
                    │  compute    │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  offset     │  parallel offset curves
                    │  curves     │  with miter joints
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  triangulate│  round caps at both ends
                    │  mesh       │  fill triangles output
                    └─────────────┘
```

### Width Formula

Per-segment width is computed from the pen pressure and drawing speed:

```
width = base_size × (0.15 + 0.85 × pressure × (1.0 − 0.75 × speed))
```

- `base_size` — user's selected brush size (1–20)
- `pressure` — pen pressure from pointer event (0.0–1.0), or 0.5 for mouse
- `speed` — normalized segment velocity (distance / time, clamped 0–1)

The result is tapered at both ends of the stroke for a natural look.

### Shape Meshes

Shapes are generated directly as triangle meshes in Rust, bypassing the stroke pipeline:

| Shape | Mesh construction |
|-------|-------------------|
| Rectangle | 4 triangles (around AA'BB') |
| Circle | Triangle fan, configurable segments |
| Line | Offset rectangle, round cap arcs at ends |
| Arrow | Line mesh + arrowhead triangles |

### Hit Testing

The `hit_path` function tests whether a point falls within a stroke's variable-width envelope by iterating segments and checking distance against the interpolated width at the closest point on the segment.

## Using scrib as a Rust Library

Add scrib to your `Cargo.toml`:

```toml
[dependencies]
scrib = { git = "https://github.com/Himanshu121865/scrib" }
```

For WASM targets, enable the `wasm` feature:

```toml
[dependencies]
scrib = { git = "https://github.com/Himanshu121865/scrib", features = ["wasm"] }
```

### Core Types

```rust
// A 2D point with pressure
let p = Point::new(1.0, 2.0, 0.5);
assert_eq!(p.x, 1.0);
assert_eq!(p.y, 2.0);
assert_eq!(p.pressure, 0.5);
```

```rust
// A stroke collects raw points and processes them
let mut stroke = Stroke::new();
stroke.add_point(Point::new(0.0, 0.0, 0.5));
stroke.add_point(Point::new(50.0, 80.0, 0.8));
stroke.add_point(Point::new(100.0, 100.0, 0.3));
```

### Processing Pipeline

```rust
// Run the full pipeline: simplify → smooth → compute widths
// Returns Vec<[f64; 3]> — [x, y, width] for each smoothed point
let result = stroke.process_with_widths(4.0);
```

### Step-by-Step

```rust
use scrib::{simplify::rdp, smooth::catmull_rom, stroke::compute_widths};

let raw = vec![
    Point::new(0.0, 0.0, 0.5),
    Point::new(50.0, 80.0, 0.8),
    Point::new(100.0, 100.0, 0.3),
];

// 1. Simplify with Ramer-Douglas-Peucker (epsilon = 0.3)
let simplified = rdp(&raw, 0.3);

// 2. Smooth with centripetal Catmull-Rom (8 segments per pair)
let smoothed = catmull_rom(&simplified, 8, 0.5);

// 3. Compute per-point widths
let widths = compute_widths(&smoothed, 4.0);
```

### Mesh Generation

```rust
use scrib::geometry::offset_curve_to_mesh;

let points = vec![
    Point::new(0.0, 0.0, 0.5),
    Point::new(100.0, 0.0, 0.5),
];
let widths = vec![2.0, 4.0];

// Returns Vec<f64> — flat triangle vertex array [x, y, x, y, ...]
let mesh = offset_curve_to_mesh(&points, &widths, false);
```

### Pipeline Summary

```
raw points → RDP simplify → Catmull-Rom smooth → width computation → output
```

Centripetal Catmull-Rom interpolation (`alpha = 0.5`) produces smooth curves through all input points. Width is computed per-segment from velocity and pressure:

```
width = base_size × (0.15 + 0.85 × pressure × (1.0 − 0.75 × speed))
```

### WASM Exports

| Function | Returns | Description |
|----------|---------|-------------|
| `process_stroke(raw, epsilon, segments, size)` | `Float64Array` | Full pipeline: raw coords → `[x, y, width, ...]` |
| `mesh_from_centerline(data)` | `Float64Array` | Offset curves → triangulated `[x, y, ...]` |
| `shape_mesh(kind, x1, y1, x2, y2, width, segs)` | `Float64Array` | Shape triangle mesh (rect/circle/line/arrow) |
| `hit_path(px, py, data, width)` | `bool` | Point-in-stroke hit test |

## Algorithmic Details

### Ramer-Douglas-Peucker Simplification

RDP reduces the number of points by removing those that deviate less than `epsilon` from a straight-line approximation:

```
1. Take the first and last point as a segment
2. Find the point farthest from this segment
3. If its distance > ε, recurse on both sub-segments
4. Otherwise, discard all intermediate points
```

In scrib, `epsilon = 0.3` provides a good balance between shape preservation and point reduction. A typical 200-point raw stroke is reduced to ~30–50 points.

### Centripetal Catmull-Rom Spline

Catmull-Rom is a cubic Hermite spline that passes through all control points. The centripetal variant (`α = 0.5`) parametrizes the curve by the square root of chord length, which prevents cusps and self-intersections:

```
For four control points P0, P1, P2, P3:
  t0 = 0
  t1 = |P1 - P0|^α + t0
  t2 = |P2 - P1|^α + t1
  t3 = |P3 - P2|^α + t2

  For a given t in [t1, t2]:
    A1 = (t1 - t)/(t1 - t0) × P0 + (t - t0)/(t1 - t0) × P1
    A2 = (t2 - t)/(t2 - t1) × P1 + (t - t1)/(t2 - t1) × P2
    A3 = (t3 - t)/(t3 - t2) × P2 + (t - t2)/(t3 - t2) × P3
    B1 = (t2 - t)/(t2 - t0) × A1 + (t - t0)/(t2 - t0) × A2
    B2 = (t3 - t)/(t3 - t1) × A2 + (t - t1)/(t3 - t1) × A3
    C  = (t2 - t)/(t2 - t1) × B1 + (t - t1)/(t2 - t1) × B2
```

Each segment (P1–P2) produces 8 interpolated points by default, resulting in smooth curves even from sparse simplified input.

### Offset Curves

For variable-width strokes, each side of the centerline is offset by half the local width:

```
left  = point + normal × (width / 2)
right = point - normal × (width / 2)
```

At sharp turns, miter joints extend the offset curves until they intersect. If the miter limit is exceeded, a bevel joint is used instead.

### Normal and Tangent Vectors

For a segment from point A to point B:

```
direction = normalize(B - A)
normal    = (-direction.y, direction.x)  (perpendicular, pointing left)
```

The normal is used to offset points left/right for the variable-width envelope. The tangent is the direction vector, used for cap arc placement.

### Miter Joints

When two consecutive segments form an angle, the offset curves on the inner side would overlap. Miter joints extend both offset lines until they intersect:

```
For segments AB and BC with offset distance d:
  left_A  = A + normal_AB × d
  left_B  = B + normal_AB × d
  left_B2 = B + normal_BC × d

  intersection = line_intersection(left_A→left_B, left_B2→left_BC)
```

If the miter length exceeds `miter_limit × d`, a bevel joint is used instead (connecting the offset endpoints directly).

### Capping

Both ends of a stroke are capped with a semicircular arc of triangles:

```
For N cap segments:
  angle = i × π / N  (i = 0..N)
  cap_point = end + normal × cos(angle) + tangent × sin(angle)
```

The cap center is the endpoint, and the arc sweeps from one side to the other.

### Triangulation

The offset left/right curves, together with the caps, form a closed polygon. This polygon is triangulated using a simple ear-clipping algorithm adapted for convex shapes:

```
left_curve (reversed) + cap2 + right_curve + cap1
→ closed polygon
→ triangle fan from first vertex
```

The resulting triangle mesh is returned as a flat `Float64Array` for direct Canvas2D consumption.

## Client State Management

All mutable application state lives in a single object `S` exported from `state.js`:

```javascript
export const S = {
  mode: 'draw',
  tool: 'pen',
  color: '#000000',
  size: 4,
  bg: '#eee',
  dark: false,
  zoom: 1,
  offsetX: 0, offsetY: 0,
  grid: false,
  strokes: [],
  shapes: [],
  redoStack: [],
  // … more state
};
```

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| Single mutable object | ES module imports are read-only bindings; reassigning `export let x` from another module doesn't work. Mutating `S.x` does. |
| No framework | Zero JS dependencies. Canvas2D + vanilla JS is sufficient for this scope. |
| Global state | Simple and direct. The app is small enough that reactivity isn't needed. |

### State Persistence

State is NOT persisted across page loads. Undo/redo operates on an in-memory 100-level stack via `saveState()` which deep-copies `S.strokes` and `S.shapes` before each mutation.

### State Reset Triggers

| Event | What resets |
|-------|-------------|
| Tool switch | Incremental cache (`incrCache`) |
| Stroke start | Incremental cache |
| Clear | All strokes, shapes, undo/redo stacks |
| Undo/Redo | Incremental cache, then restores from stack |

## Dependency Graph

The project is designed with minimal dependencies. Here's what depends on what:

```text
scrib (workspace root)
  ├── src/ (core library)
  │     └── zero dependencies in Rust
  │     └── wasm feature → wasm-bindgen (optional)
  │
  ├── server/
  │     ├── tokio          — async runtime
  │     ├── tokio-tungstenite — WebSocket server
  │     ├── serde / serde_json — JSON serialization
  │     └── futures-util   — async stream utilities
  │
  └── www/ (no build step, no bundler)
        └── zero JS dependencies
        └── scrib.js + scrib_bg.wasm (from wasm-pack)
```

### Why Zero Rust Dependencies in Core?

The core library (`src/`) deliberately avoids external dependencies to remain portable and easy to audit. All math — vector operations, Catmull-Rom interpolation, RDP simplification, offset curves, triangulation — is implemented from scratch. This also keeps compile times fast and eliminates supply-chain risk for the core algorithm.

### Why Tokio in the Server?

The server needs concurrent WebSocket handling for multiple rooms and clients. Tokio is the standard async runtime in Rust, and tokio-tungstenite provides battle-tested WebSocket support. These are dev-dependencies of the server binary only — they don't affect library consumers.

## Client Rendering

The browser client renders strokes and shapes using Canvas2D with triangle meshes generated by the Rust WASM module.

### Rendering Pipeline

```text
pointer event → collect points → tools.js
  │
  ├── freehand: processStrokeIncremental()
  │     │
  │     ├── wasm.process_stroke() → centerline [x, y, width, ...]
  │     ├── wasm.mesh_from_centerline() → Float64Array
  │     └── redraw() → drawMesh() → ctx.fill(triangles)
  │
  └── shape: wasm.shape_mesh() → Float64Array
        └── redraw() → drawMesh() → ctx.fill(triangles)
```

### drawMesh

Renders a triangle mesh to the canvas in a single `fill()` call:

```javascript
function drawMesh(ctx, mesh, color) {
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.moveTo(mesh[0], mesh[1]);
  for (let i = 2; i < mesh.length; i += 2) {
    // every 6th vertex starts a new triangle (moveTo)
    if (i % 6 === 0) ctx.moveTo(mesh[i], mesh[i+1]);
    else ctx.lineTo(mesh[i], mesh[i+1]);
  }
  ctx.closePath();
  ctx.fill();
}
```

Every 6 floats (3 vertices) marks a complete triangle, so `moveTo` is called at the start of each triangle while `lineTo` fills in the remaining two vertices. This renders the entire stroke in a single draw call.

### Cursor Overlay

A separate canvas (`#cursorCanvas`) with `pointer-events: none` and `z-index: 5` sits on top of the main canvas. Remote cursor positions are drawn here via `redrawCursors()`, which only redraws cursors — never the full scene. This decouples cursor updates from stroke rendering.

### Animation Loop

`tickAnimation()` runs via `requestAnimationFrame` and handles:

1. **Remote cursor lerp** — target `tx, ty` approaches render `rx, ry` at 25% per frame
2. **Remote shape lerp** — shape endpoints lerp at 25% per frame, mesh regenerates each frame
3. **Snap threshold** — when distance < 0.5px, render snaps directly to target for precision

## Performance

### WASM Mesh Generation

All geometry processing happens in Rust compiled to WASM, running off the main thread (via synchronous calls that complete within 1–5ms for typical strokes). The heaviest operation — triangulation — produces a flat `Float64Array` that JavaScript copies directly to the canvas.

### Incremental Processing

Long strokes are broken into chunks. Only the last 4 segments are re-meshed on each pointer move (30ms throttle). This keeps frame times below 16ms even during rapid drawing.

### Rendering Path

```
mesh (Float64Array) → Canvas2D fill() with a single path
```

A stroke with 100 smoothed points produces ~200 triangles → ~600 floats → one `ctx.fill()` call. No per-segment draw calls, no SVG overhead.

### Memory

Server stores strokes as JSON strings in memory. A typical stroke is ~5KB. At 100 strokes per room, that's ~500KB per room — negligible for modern hardware.

## Testing

### Rust Unit Tests

The core library has 22 unit tests across four modules:

```
$ cargo test
   Compiling scrib v0.1.0
    Finished test [unoptimized + debuginfo]
    
running 22 tests
test geometry::tests::test_closed_path_mesh    ... ok
test geometry::tests::test_hit_path            ... ok
test geometry::tests::test_offset_curve        ... ok
test geometry::tests::test_round_caps          ... ok
test geometry::tests::test_shape_meshes        ... ok
test simplify::tests::test_corner_preservation ... ok
test simplify::tests::test_rdp_basic           ... ok
test simplify::tests::test_rdp_empty           ... ok
test simplify::tests::test_rdp_two_points      ... ok
test simplify::tests::test_rdp_vertical_line   ... ok
test smooth::tests::test_centripetal           ... ok
test smooth::tests::test_curve_matches_centroid ... ok
test smooth::tests::test_linear_points         ... ok
test smooth::tests::test_smooth_coincident     ... ok
test smooth::tests::test_smooth_empty          ... ok
test smooth::tests::test_smooth_three_points   ... ok
test stroke::tests::test_add_point             ... ok
test stroke::tests::test_process_empty         ... ok
test stroke::tests::test_process_single        ... ok
test stroke::tests::test_taper                 ... ok
test stroke::tests::test_width_range           ... ok
test stroke::tests::test_widths_basic          ... ok
test result: ok. 22 passed; 0 failed
```

### Test Categories

| Module | Tests | What's covered |
|--------|-------|----------------|
| `simplify` | 4 | Basic RDP, empty/2-point edge cases, corner preservation, vertical lines |
| `smooth` | 6 | Centripetal vs uniform, linear points, coincident points, empty/3-point edge cases, curve centroid |
| `stroke` | 6 | Adding points, empty/single edge cases, width computation, tapering, pressure-sensitive range |
| `geometry` | 6 | Offset curves, round caps, closed paths, shape meshes, hit testing |

### Running Tests

```bash
# All tests
cargo test

# Single module
cargo test simplify
cargo test smooth
cargo test stroke
cargo test geometry

# Single test
cargo test test_rdp_basic

# With output
cargo test -- --nocapture
```

## Architecture

```
┌──────────────────────────────────────────────────┐
│  www/  (vanilla JS + Canvas2D)                   │
│  ├── app.js       entry, UI, pointer events      │
│  ├── state.js     shared mutable state           │
│  ├── render.js    redraw, grid, cursors          │
│  ├── tools.js     draw/erase/shape handlers      │
│  ├── network.js   WebSocket message handlers     │
│  ├── style.css                                   │
│  └── index.html   skeleton                       │
├──────────────────────────────────────────────────┤
│  pkg/   (compiled WASM — scrib.d.ts, scrib.js)   │
├──────────────────────────────────────────────────┤
│  src/   (Rust library, zero deps)                │
│  ├── point.rs       Point { x, y, pressure }     │
│  ├── simplify.rs    Ramer-Douglas-Peucker        │
│  ├── smooth.rs      Catmull-Rom interpolation    │
│  ├── stroke.rs      pipeline + width computation │
│  ├── geometry.rs    offset curves, mesh, shapes  │
│  └── wasm.rs        wasm-bindgen bindings        │
├──────────────────────────────────────────────────┤
│  server/ (Rust, tokio-tungstenite)               │
│  └── main.rs        rooms, strokes, broadcast    │
└──────────────────────────────────────────────────┘
```

## Server Architecture

The multiplayer server (`server/src/main.rs`) is a lightweight Rust binary using `tokio-tungstenite` for async WebSocket handling.

```
┌───────────────────────────────────────────────┐
│  Server (:9876)                                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ Room A   │  │ Room B   │  │ Room C   │    │
│  │ ┌──────┐ │  │ ┌──────┐ │  │ ┌──────┐ │    │
│  │ │user1 │ │  │ │user4 │ │  │ │user7 │ │    │
│  │ │user2 │ │  │ │user5 │ │  │ │user8 │ │    │
│  │ │user3 │ │  │ │user6 │ │  │ └──────┘ │    │
│  │ └──────┘ │  │ └──────┘ │                 │
│  │ strokes  │  │ strokes  │                 │
│  │ buf: 12  │  │ buf: 5   │                 │
│  └──────────┘  └──────────┘                 │
└───────────────────────────────────────────────┘
```

### Key Design Decisions

- **In-memory storage** — rooms and strokes live in `Arc<Mutex<HashMap>>`. No persistence layer. Late joiners receive all stored strokes on connect.
- **Broadcast** — messages are fanned out to every connected client in the room except the sender.
- **ID migration** — if a stroke arrives without an `id`, the server assigns one (`srv_<room>_<n>`). Legacy stored strokes get migrated on join.
- **No auth** — user IDs are assigned per-connection. No login, no sessions.

## Incremental Rendering

Freehand strokes are processed incrementally to keep the UI responsive during long strokes.

```
pen down  → cache: centerline=[], mesh=null
pen move  → if 30ms elapsed:
             └─ smooth new points → append centerline
             └─ regenerate mesh from last N segments
pen up    → finalize: regenerate entire mesh
             └─ cache: centerline + mesh
```

The 30ms throttle prevents WASM calls from blocking the main thread. Only the tail portion of the mesh (the last 4 segments) is regenerated on each move. The cache is reset on stroke start/end, undo, redo, clear, and tool switch.

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SCR_WS_PORT` | `9876` | WebSocket server port |
| `SCR_WS_HOST` | `0.0.0.0` | WebSocket server bind address |
| `RUST_LOG` | — | Server log level (`info`, `debug`, `warn`) |

The server reads these at startup. The client always connects to `ws://<hostname>:9876`.

### WASM Build Features

| Feature | Default | Description |
|---------|---------|-------------|
| `wasm` | off | Enables wasm-bindgen bindings for browser use |
| default | on | Pure Rust library, no WASM dependency |

The `wasm` feature is required when building with `wasm-pack` but NOT needed when using scrib as a native Rust library.

## Development

### Prerequisites

| Tool | Install |
|------|---------|
| Rust | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` |
| wasm-pack | `cargo install wasm-pack` |
| Python 3 | `apt install python3` / `brew install python` |

### Commands

| Command | Description |
|---------|-------------|
| `cargo test` | Run all unit tests (22 tests) |
| `cargo clippy` | Lint |
| `cargo fmt` | Format |
| `cargo run -p scrib-server` | Start WebSocket server on `:9876` |
| `wasm-pack build --target web --features wasm` | Rebuild WASM |
| `cp pkg/* www/pkg/` | Copy WASM artifacts to web demo |
| `python3 -m http.server 8080` | Serve web demo |

### Build Pipeline

```
source (Rust)
  ├── cargo build           → native binary (tests)
  ├── cargo run -p scrib-server → server binary
  └── wasm-pack build
        └── pkg/
              ├── scrib_bg.wasm
              ├── scrib.js
              └── scrib.d.ts
                    └── copied to www/pkg/
```

### Project Structure

| Path | Purpose |
|------|---------|
| `src/` | Core Rust library — geometry, simplify, smooth, stroke pipeline |
| `server/` | WebSocket server binary (tokio-tungstenite) |
| `www/` | Browser client — vanilla JS modules, Canvas2D rendering |
| `www/style.css` | All UI styles |
| `www/index.html` | HTML skeleton |
| `www/pkg/` | Built WASM artifacts (gitignored) |
| `Cargo.toml` | Workspace root, `wasm` feature flag |
| `server/Cargo.toml` | Server dependencies |
| `AGENTS.md` | AI assistant development notes |

### Browser Client Modules

| File | Responsibility |
|------|---------------|
| `app.js` | Entry point, WASM init, undo/redo, UI events, pointer handlers |
| `state.js` | Constants, mutable state object `S`, pure utility functions |
| `render.js` | Canvas redraw, grid, cursor overlay, animation loop |
| `tools.js` | Drawing/erasing/shape handlers, incremental processing |
| `network.js` | WebSocket init, send/receive message handlers |

### UI Layout

```
┌─────────────────────────────────────────────────────┐
│  Top Bar: undo | redo | clear | zoom+-fit | grid |  │
│           bg toggle | help                           │
├────┬────────────────────────────────────────────────┤
│    │                                                 │
│ T  │           Canvas                                │
│ o  │     (infinite scroll/pan, cursor overlay)      │
│ o  │                                                 │
│ l  │                                                 │
│ s  │                                                 │
│    │                                                 │
├────┴────────────────────────────────────────────────┤
│  Bottom Bar: color swatches | custom + | size slider │
└─────────────────────────────────────────────────────┘
```

## Troubleshooting

### WASM build fails

Ensure `wasm-pack` is installed and you're passing `--features wasm`:

```bash
wasm-pack build --target web --features wasm
```

### WebSocket connection refused

The server must be running before opening the web demo:

```bash
cargo run -p scrib-server
```

Default port is `9876`. The client connects to `ws://{hostname}:9876`.

### Strokes not visible to other users

- Verify all clients are in the same room (same URL hash)
- Check the server logs for connection events
- Refresh the browser after the server starts

### Canvas appears blank

Make sure `www/pkg/` contains the built WASM files. Run the build step:

```bash
wasm-pack build --target web --features wasm && cp pkg/* www/pkg/
```

### WASM initialization error

Check the browser console for specific errors. Common causes:
- Outdated `wasm-bindgen` version (run `wasm-pack build` again)
- Missing `--target web` flag
- Browser doesn't support WASM (check `about:config`)

### Performance issues with long strokes

The incremental processing pipeline is designed to handle long strokes. If you experience lag:
- Reduce the brush size (smaller meshes)
- Increase the throttle interval in `tools.js::processStrokeIncremental`
- Close unused browser tabs sharing the same room

## FAQ

**Q: Does scrib store strokes on disk?**
A: No. The server keeps strokes in memory only. Closing the server loses all drawings.

**Q: Can I use scrib offline / single-player?**
A: Yes. Just open `www/index.html` without starting the server. Multiplayer features will be unavailable but drawing works.

**Q: What pointer devices are supported?**
A: Mouse, touch, and pressure-sensitive pens. Pressure data is used for variable-width strokes.

**Q: How do I change the room name after connecting?**
A: Refresh the page with a different URL hash. All previous strokes in the old room remain there.

**Q: Is there a save/export feature?**
A: Not yet. Strokes exist only in the server's memory during the session.

**Q: What browsers are supported?**
A: Any modern browser with WASM support: Chrome, Firefox, Safari, Edge.

## Contributing

### Getting Started

1. Fork the repository on GitHub
2. Clone your fork: `git clone git@github.com:YOUR_USERNAME/scrib.git`
3. Create a feature branch: `git checkout -b feature/my-change`
4. Make your changes
5. Run `cargo test && cargo clippy` to verify everything passes
6. Commit with a descriptive message
7. Push and submit a pull request

### Coding Standards

- Rust: follow `cargo clippy` suggestions, use `cargo fmt` before committing
- JavaScript: ES modules, no external dependencies, single mutable state object `S`
- No comments in source code (keep explanations in commit messages and PRs)
- Match existing code style — look at surrounding code before making changes

### What Needs Help

- Save/export (PNG, SVG)
- Undo/redo for shapes
- Stroke smoothing parameters in the UI
- Server persistence (SQLite)
- Mobile touch optimization
- Pressure sensitivity tuning

## Comparison

### vs Excalidraw, tldraw, Rough.js

| Feature | scrib | Excalidraw | tldraw | Rough.js |
|---------|-------|------------|--------|----------|
| Variable width strokes | ✅ velocity+pressure | ✅ pressure | ❌ fixed | ❌ fixed |
| WASM core | ✅ Rust → WASM | ❌ plain JS | ❌ plain JS | ❌ plain JS |
| Zero JS dependencies | ✅ | ❌ | ❌ | ❌ |
| Offline drawing | ✅ standalone | ❌ requires backend | ❌ requires backend | ✅ standalone |
| Shapes | ✅ rect, circle, line, arrow | ✅ extensive | ✅ extensive | ✅ extensive |
| Multiplayer | ✅ built-in WS server | ✅ via backend | ✅ via backend | ❌ |
| Infinite canvas | ✅ | ✅ | ✅ | ❌ |
| Grid snap | ✅ | ✅ | ✅ | ❌ |
| Undo/redo | ✅ 100 levels | ✅ | ✅ | ❌ |
| Eraser | ✅ hit-testing | ✅ | ✅ | ❌ |
| Export | ❌ planned | ✅ PNG/SVG | ✅ PNG/SVG | ✅ SVG |
| Pressure sensitivity | ✅ | ✅ | ❌ | ❌ |
| Bundle size | ~40KB WASM | ~200KB JS | ~300KB JS | ~20KB JS |
| Browser support | modern (WASM) | all modern | all modern | all modern |

### Why scrib over the alternatives?

| Need | Recommendation |
|------|---------------|
| **Embedded drawing** (no npm, no bundler) | scrib — zero JS deps, `<script type="module">` only |
| **Multiplayer whiteboard** | scrib — built-in WebSocket server, no backend setup |
| **Pressure-sensitive strokes** | scrib — velocity + pressure width (Excalidraw also good) |
| **Rough/hand-drawn aesthetic** | Rough.js — scrib is smooth, not rough |
| **Rich shape library** | Excalidraw / tldraw — scrib has 4 shapes |
| **Production collaboration** | Excalidraw — mature, feature-rich |

## Future Work

- [ ] Save/export to PNG and SVG
- [ ] Undo/redo support for shapes
- [ ] Adjustable smoothing parameters in UI
- [ ] Server persistence (SQLite/JSON file)
- [ ] Mobile touch optimization
- [ ] Pressure sensitivity calibration UI
- [ ] Text tool
- [ ] Image embedding
- [ ] Layers
- [ ] Selection + move/resize
- [ ] Copy/paste between rooms

## Data Model

All strokes and shapes share a common JSON structure for serialization over WebSocket and storage.

### Stroke Object

```json
{
  "id": "srv_roomA_3",
  "userId": "user_abc123",
  "kind": "path",
  "data": [0.0, 0.0, 0.5, 12.3, 45.6, 0.5, ...],
  "color": "#e86a20",
  "width": 4.0,
  "mesh": [0.0, 0.0, 10.0, 0.0, ...]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique stroke ID |
| `userId` | string | Creator's session ID |
| `kind` | `"path"` | Always `"path"` for freehand strokes |
| `data` | Float64Array | Interleaved `[x, y, width, x, y, width, ...]` |
| `color` | string | Hex color |
| `width` | number | Base brush width |
| `mesh` | Float64Array | Triangle mesh `[x, y, x, y, ...]` |

### Shape Object

```json
{
  "id": "srv_roomA_7",
  "userId": "user_def456",
  "kind": "rect",
  "x1": 10.0, "y1": 20.0,
  "x2": 100.0, "y2": 200.0,
  "color": "#000000",
  "width": 2.0,
  "mesh": [10.0, 20.0, 100.0, 20.0, ...]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `kind` | string | `"rect"`, `"circle"`, `"line"`, `"arrow"` |
| `x1,y1` | number | Start/anchor point |
| `x2,y2` | number | End point |
| `mesh` | Float64Array | Generated on the fly, not always stored |

### Storage

The server stores strokes as JSON strings in `Arc<Mutex<HashMap<String, Vec<String>>>>` keyed by room name. On client join, all stored strokes are sent as `stroke-end` messages. No persistence to disk.

## Changelog

### 2026-07 — Current

- Incremental stroke processing with centerline + mesh caching
- Cursor overlay canvas for efficient cursor redraws
- Shape mesh generators (rect, circle, line, arrow)
- Hit testing in Rust via `hit_path`
- Erase ownership enforcement (server + client)
- Remote cursor and shape smoothing via lerp
- Infinite canvas (zoom/pan)
- Multiplayer server with room management
- WebSocket protocol with stroke streaming
- Refactored monolith into ES modules (state.js, render.js, tools.js, network.js, app.js)
- All source comments removed
- WASM build with wasm-bindgen

### 2026-06

- Core Rust library complete: Point, Stroke, RDP simplify, Catmull-Rom smooth, offset curves, triangulation
- WASM bindings: process_stroke, mesh_from_centerline
- Basic web demo with freehand drawing
- Initial server with tokio-tungstenite

### 2026-05

- Project inception
- Zero-dependency Rust math library started

## Security

### WebSocket Connections

The server uses plain WebSocket (`ws://`) with no TLS. For local/trusted network use, this is sufficient. For production deployments, wrap the server behind a reverse proxy (nginx, Caddy) with TLS termination.

### User Identity

There is no authentication. User IDs are random strings generated by the server on connection. Anyone who can connect to the WebSocket port can join any room. There is no access control beyond the network level.

### Data Isolation

Rooms are isolated in-memory. Clients in room A cannot see strokes or cursors from room B. The server validates all incoming messages against the client's assigned room.

### Erase Authorization

The server enforces that a client can only erase strokes where `stroke.user_id == client.user_id`. This is checked server-side before the erase is processed or broadcast.

### Client-Side Safety

The browser client runs WASM compiled from Rust — no eval, no dynamic code execution. All coordinate transforms happen in pure math. The only external network connection is the WebSocket to the scrib server.

## License

MIT — see [LICENSE](LICENSE) for details.


hello
