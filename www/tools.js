import { process_stroke, mesh_from_centerline, shape_mesh, hit_path } from './pkg/scrib.js';
import { S, canvas, cursorCanvas, EPSILON, SEGMENTS, CAP_FLOATS, INCR_THROTTLE, THROTTLE_DRAW, GRID, saveState, snap, distToSegment } from './state.js';
import { redraw } from './render.js';

export function hitStroke(px, py) {
  for (let i = S.strokes.length - 1; i >= 0; i--) {
    if (S.toErase.has(i)) continue;
    const s = S.strokes[i];
    const t = Math.max(s.size / 2, 4);
    if (s.type === 'dot') {
      if (Math.hypot(px - s.x, py - s.y) <= t) return i;
    } else if (s.type === 'path') {
      if (s.data && hit_path(px, py, s.data, s.size)) return i;
    } else if (s.type === 'rect') {
      const l = Math.min(s.x1, s.x2), r = Math.max(s.x1, s.x2);
      const u = Math.min(s.y1, s.y2), d = Math.max(s.y1, s.y2);
      if (Math.hypot(px - Math.max(l, Math.min(r, px)), py - Math.max(u, Math.min(d, py))) <= t) return i;
    } else if (s.type === 'circle') {
      const cx = (s.x1 + s.x2) / 2, cy = (s.y1 + s.y2) / 2;
      if (Math.abs(Math.hypot(px - cx, py - cy) - Math.hypot(s.x2 - s.x1, s.y2 - s.y1) / 2) <= t) return i;
    } else if (s.type === 'line' || s.type === 'arrow') {
      if (distToSegment(px, py, s.x1, s.y1, s.x2, s.y2) <= t) return i;
    }
  }
  return -1;
}

export function cancelErase() {
  S.erasing = false;
  S.toErase = new Set();
  S.eraserPath = [];
  redraw();
}

export function resetIncrCache() {
  S.cachedCenterline = null;
  S.cachedMesh = null;
}

export function processStrokeIncremental(raw) {
  const cl = Array.from(process_stroke(raw, EPSILON, SEGMENTS, S.currentSize));

  if (S.cachedCenterline && S.cachedMesh && S.cachedCenterline.length >= 6) {
    let diffIdx = 0;
    const minLen = Math.min(S.cachedCenterline.length, cl.length);
    while (diffIdx + 2 < minLen &&
           cl[diffIdx] === S.cachedCenterline[diffIdx] &&
           cl[diffIdx + 1] === S.cachedCenterline[diffIdx + 1]) {
      diffIdx += 3;
    }

    if (diffIdx >= 6) {
      const overlapIdx = (diffIdx / 3) | 0;
      const keepSegs = overlapIdx - 1;
      const tail = cl.slice((overlapIdx - 1) * 3);
      const tailMesh = Array.from(mesh_from_centerline(tail));

      const bodyStart = CAP_FLOATS;
      const keepLen = keepSegs > 0 ? keepSegs * 12 : 0;
      const merged = S.cachedMesh.slice(0, bodyStart + keepLen)
        .concat(tailMesh.slice(CAP_FLOATS));

      S.cachedCenterline = cl;
      S.cachedMesh = merged;
      return { data: cl, mesh: merged };
    }
  }

  const mesh = Array.from(mesh_from_centerline(cl));
  S.cachedCenterline = cl;
  S.cachedMesh = mesh;
  return { data: cl, mesh: mesh };
}

export function finalizeStroke() {
  if (!S.currentRaw) return;
  saveState();
  let obj;
  if (S.currentRaw.length === 3) {
    obj = { type: 'dot', x: S.currentRaw[0], y: S.currentRaw[1], color: S.currentColor, size: S.currentSize, pressure: S.currentRaw[2], id: S.currentStrokeId, userId: S.myId };
    S.strokes.push(obj);
  } else if (S.currentRaw.length > 3) {
    const data = Array.from(process_stroke(S.currentRaw, EPSILON, SEGMENTS, S.currentSize));
    const mesh = Array.from(mesh_from_centerline(data));
    obj = { type: 'path', data, mesh, color: S.currentColor, size: S.currentSize, id: S.currentStrokeId, userId: S.myId };
    S.strokes.push(obj);
  }
  if (S.ws?.readyState === WebSocket.OPEN && S.currentStrokeId) {
    S.ws.send(JSON.stringify({type: 'stroke-end', data: {id: S.currentStrokeId, stroke: obj}}));
  }
  S.currentRaw = null;
  S.currentStrokeId = null;
  resetIncrCache();
}

export function handleDown(x, y, pressure) {
  if (S.currentTool === 'eraser') {
    S.erasing = true;
    S.eraserPath = [x, y];
    S.toErase = new Set();
    return;
  }
  if (S.currentTool === 'draw') {
    S.drawing = true;
    S.currentRaw = [x, y, pressure];
    S.currentStrokeId = Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
    S.cachedMesh = null;
    resetIncrCache();
    S.lastStrokeSendTime = 0;
    if (S.ws?.readyState === WebSocket.OPEN) {
      S.ws.send(JSON.stringify({type: 'stroke-start', data: {id: S.currentStrokeId, color: S.currentColor, size: S.currentSize}}));
    }
    return;
  }
  S.shaping = true;
  S.currentShapeId = 'shape_' + Date.now().toString(36) + Math.random().toString(36).slice(2, 5);
  S.shapeStart = { x: snap(x), y: snap(y), x2: snap(x), y2: snap(y) };
}

export function handleMove(x, y) {
  if (S.currentTool === 'eraser') {
    if (!S.erasing) return;
    S.eraserPath.push(x, y);
    const idx = hitStroke(x, y);
    if (idx >= 0) S.toErase.add(idx);
    if (S.eraserPath.length >= 4) {
      const mx = (S.eraserPath[S.eraserPath.length - 4] + x) / 2;
      const my = (S.eraserPath[S.eraserPath.length - 3] + y) / 2;
      const idx2 = hitStroke(mx, my);
      if (idx2 >= 0) S.toErase.add(idx2);
    }
    redraw();
    return;
  }
  if (S.currentTool === 'draw') {
    if (!S.drawing) return;
    S.currentRaw.push(x, y, 0.5);
    const now = Date.now();
    if (S.currentRaw.length > 3 && now - S.lastIncrTime > INCR_THROTTLE) {
      S.lastIncrTime = now;
      const { mesh } = processStrokeIncremental(S.currentRaw);
      S.cachedMesh = mesh;
    }
    if (now - S.lastStrokeSendTime > THROTTLE_DRAW && S.ws?.readyState === WebSocket.OPEN && S.currentStrokeId && S.cachedMesh) {
      S.lastStrokeSendTime = now;
      S.ws.send(JSON.stringify({type: 'stroke-update', data: {id: S.currentStrokeId, mesh: S.cachedMesh}}));
    }
    redraw();
    return;
  }
  if (!S.shaping) return;
  S.shapeStart.x2 = snap(x);
  S.shapeStart.y2 = snap(y);
  const now = Date.now();
  if (now - S.lastStrokeSendTime > THROTTLE_DRAW && S.ws?.readyState === WebSocket.OPEN && S.currentShapeId) {
    S.lastStrokeSendTime = now;
    S.ws.send(JSON.stringify({type: 'shape-update', data: {
      id: S.currentShapeId,
      shape: { type: S.currentTool, x1: S.shapeStart.x, y1: S.shapeStart.y, x2: S.shapeStart.x2, y2: S.shapeStart.y2, color: S.currentColor, size: S.currentSize }
    }}));
  }
  redraw();
}

export function handleUp() {
  if (S.currentTool === 'eraser') {
    if (!S.erasing) return;
    S.erasing = false;
    saveState();
    const sorted = [...S.toErase].sort((a, b) => b - a);
    const erasedIds = [];
    for (const idx of sorted) {
      const s = S.strokes[idx];
      if (!s) continue;
      if (s.userId !== undefined && s.userId !== S.myId) continue;
      if (s.id) erasedIds.push(s.id);
      S.strokes.splice(idx, 1);
    }
    if (erasedIds.length > 0) {
      try {
        if (S.ws?.readyState === WebSocket.OPEN) {
          S.ws.send(JSON.stringify({type: 'erase', ids: erasedIds}));
        }
      } catch(e) { console.error('erase send failed', e); }
    }
    S.toErase = new Set();
    S.eraserPath = [];
    redraw();
    return;
  }
  if (S.currentTool === 'draw') {
    if (!S.drawing) return;
    S.drawing = false;
    finalizeStroke();
    redraw();
    return;
  }
  if (!S.shaping) return;
  S.shaping = false;
  if (Math.abs(S.shapeStart.x2 - S.shapeStart.x) > 2 || Math.abs(S.shapeStart.y2 - S.shapeStart.y) > 2) {
    saveState();
    const mesh = Array.from(shape_mesh(S.currentTool, S.shapeStart.x, S.shapeStart.y, S.shapeStart.x2, S.shapeStart.y2, S.currentSize, 32));
    const obj = { type: S.currentTool, x1: S.shapeStart.x, y1: S.shapeStart.y, x2: S.shapeStart.x2, y2: S.shapeStart.y2, mesh, color: S.currentColor, size: S.currentSize, id: S.currentShapeId, userId: S.myId };
    S.strokes.push(obj);
    if (S.ws?.readyState === WebSocket.OPEN) {
      S.ws.send(JSON.stringify({type: 'stroke-end', data: {id: S.currentShapeId, stroke: obj}}));
    }
  }
  S.shapeStart = null;
  S.currentShapeId = null;
  redraw();
}

export function resize() {
  const w = window.innerWidth - 48;
  const h = window.innerHeight - 44 - 52;
  canvas.width = w; canvas.height = h;
  cursorCanvas.width = w; cursorCanvas.height = h;
  redraw();
}
