import { process_stroke, mesh_from_centerline, shape_mesh, hit_path, hit_shape, get_bounds, finalize_stroke, transform_move, transform_resize, regenerate_mesh } from './pkg/scrib.js';
import { S, canvas, cursorCanvas, EPSILON, SEGMENTS, CAP_FLOATS, INCR_THROTTLE, THROTTLE_DRAW, saveState, snap, getBounds, getHandles } from './state.js';
import { redraw } from './render.js';
import { sendWS } from './network.js';

function hitStroke(px, py) {
  for (let i = S.strokes.length - 1; i >= 0; i--) {
    if (S.toErase.has(i)) continue;
    const s = S.strokes[i];
    if (s.type === 'path') {
      if (s.data && hit_path(px, py, s.data, s.size)) return i;
    } else if (hit_shape(px, py, s.type, s.x||0, s.y||0, s.x2||0, s.y2||0, s.size)) {
      return i;
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

function processStrokeIncremental(raw) {
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

function finalizeStroke() {
  if (!S.currentRaw) return;
  saveState();
  const obj = finalize_stroke(S.currentRaw, S.currentColor, S.currentSize, S.currentStrokeId || '', String(S.myId), EPSILON, SEGMENTS);
  if (!obj || !obj.type) return;
  S.strokes.push(obj);
  if (S.currentStrokeId) {
    sendWS(JSON.stringify({type: 'stroke-end', data: {id: S.currentStrokeId, stroke: obj}}));
  }
  S.currentRaw = null;
  S.currentStrokeId = null;
  resetIncrCache();
}

export function handleDown(x, y, pressure) {
  if (S.currentTool === 'select') { selectHandleDown(x, y); return; }
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
    sendWS(JSON.stringify({type: 'stroke-start', data: {id: S.currentStrokeId, color: S.currentColor, size: S.currentSize}}));
    return;
  }
  S.shaping = true;
  S.currentShapeId = 'shape_' + Date.now().toString(36) + Math.random().toString(36).slice(2, 5);
  S.shapeStart = { x: snap(x), y: snap(y), x2: snap(x), y2: snap(y) };
}

export function handleMove(x, y, pressure) {
  if (S.currentTool === 'select') { selectHandleMove(x, y); return; }
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
    S.currentRaw.push(x, y, pressure ?? 0.5);
    const now = Date.now();
    if (S.currentRaw.length > 3 && now - S.lastIncrTime > INCR_THROTTLE) {
      S.lastIncrTime = now;
      const { mesh } = processStrokeIncremental(S.currentRaw);
      S.cachedMesh = mesh;
    }
    if (now - S.lastStrokeSendTime > THROTTLE_DRAW && S.currentStrokeId && S.cachedMesh) {
      S.lastStrokeSendTime = now;
      sendWS(JSON.stringify({type: 'stroke-update', data: {id: S.currentStrokeId, mesh: S.cachedMesh}}));
    }
    redraw();
    return;
  }
  if (!S.shaping) return;
  S.shapeStart.x2 = snap(x);
  S.shapeStart.y2 = snap(y);
  const now = Date.now();
  if (now - S.lastStrokeSendTime > THROTTLE_DRAW && S.currentShapeId) {
    S.lastStrokeSendTime = now;
    sendWS(JSON.stringify({type: 'shape-update', data: {
      id: S.currentShapeId,
      shape: { type: S.currentTool, x1: S.shapeStart.x, y1: S.shapeStart.y, x2: S.shapeStart.x2, y2: S.shapeStart.y2, color: S.currentColor, size: S.currentSize }
    }}));
  }
  redraw();
}

export function handleUp() {
  if (S.currentTool === 'select') { selectHandleUp(); return; }
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
      if (s.id === S.selectedId) { S.selectedId = null; S.transforming = null; }
      if (s.id) erasedIds.push(s.id);
      S.strokes.splice(idx, 1);
    }
    if (erasedIds.length > 0) {
      sendWS(JSON.stringify({type: 'erase', ids: erasedIds}));
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
    sendWS(JSON.stringify({type: 'stroke-end', data: {id: S.currentShapeId, stroke: obj}}));
  }
  S.shapeStart = null;
  S.currentShapeId = null;
  redraw();
}

function findStrokeById(id) {
  return S.strokes.find(s => s.id === id);
}

function hitHandles(px, py, handles, handleSize) {
  for (const h of handles) {
    if (Math.abs(px - h.x) <= handleSize && Math.abs(py - h.y) <= handleSize) return h.id;
  }
  return null;
}

function selectHandleDown(x, y) {
  if (S.selectedId === null) {
    const idx = hitStroke(x, y);
    if (idx >= 0) {
      S.selectedId = S.strokes[idx].id;
      redraw();
    } else {
      S.selectRect = { x1: x, y1: y, x2: x, y2: y };
    }
    return;
  }

  const sel = findStrokeById(S.selectedId);
  if (!sel) { S.selectedId = null; redraw(); return; }

  const b = getBounds(sel);
  const handles = getHandles(b);
  const handleSize = 6 / S.camZoom;
  const hit = hitHandles(x, y, handles, handleSize);

  if (hit) {
    S.transforming = { type: 'resize', handle: hit, startX: x, startY: y, b, didMove: false };
    return;
  }

  if (hitStroke(x, y) === S.strokes.indexOf(sel)) {
    S.transforming = { type: 'move', startX: x, startY: y, didMove: false };
    return;
  }

  const idx = hitStroke(x, y);
  if (idx >= 0) {
    S.selectedId = S.strokes[idx].id;
  } else {
    S.selectedId = null;
    S.selectRect = { x1: x, y1: y, x2: x, y2: y };
  }
  redraw();
}

function selectHandleMove(x, y) {
  if (S.selectRect) {
    S.selectRect.x2 = x;
    S.selectRect.y2 = y;
    redraw();
    return;
  }
  if (!S.transforming) return;
  const sel = findStrokeById(S.selectedId);
  if (!sel) return;

  if (S.transforming.type === 'move') {
    const dx = x - S.transforming.startX;
    const dy = y - S.transforming.startY;
    if (dx !== 0 || dy !== 0) S.transforming.didMove = true;
    transform_move(sel, dx, dy);
    S.transforming.startX = x; S.transforming.startY = y;
    redraw();
    return;
  }

  if (S.transforming.type === 'resize') {
    const b = S.transforming.b;
    S.transforming.didMove = true;
    transform_resize(sel, S.transforming.handle, x, y, b.x1, b.y1, b.x2, b.y2);
    redraw();
  }
}

function selectHandleUp() {
  if (S.selectRect) {
    const rx1 = Math.min(S.selectRect.x1, S.selectRect.x2);
    const ry1 = Math.min(S.selectRect.y1, S.selectRect.y2);
    const rx2 = Math.max(S.selectRect.x1, S.selectRect.x2);
    const ry2 = Math.max(S.selectRect.y1, S.selectRect.y2);
    if (Math.abs(rx2 - rx1) > 2 || Math.abs(ry2 - ry1) > 2) {
      for (let i = S.strokes.length - 1; i >= 0; i--) {
        const s = S.strokes[i];
        if (S.toErase.has(i)) continue;
        const b = getBounds(s);
        if (b.x1 < rx2 && b.x2 > rx1 && b.y1 < ry2 && b.y2 > ry1) {
          S.selectedId = s.id;
          break;
        }
      }
    } else {
      const idx = hitStroke(S.selectRect.x1, S.selectRect.y1);
      S.selectedId = idx >= 0 ? S.strokes[idx].id : null;
    }
    S.selectRect = null;
    redraw();
    return;
  }
  if (S.transforming) {
    const sel = findStrokeById(S.selectedId);
    if (S.transforming.didMove) {
      if (sel) regenerate_mesh(sel);
      if (sel) {
        sendWS(JSON.stringify({type: 'stroke-end', data: {id: sel.id, stroke: sel}}));
      }
      saveState();
    }
    S.transforming = null;
    redraw();
  }
}

export function deleteSelected() {
  if (S.selectedId === null) return;
  const idx = S.strokes.findIndex(s => s.id === S.selectedId);
  if (idx < 0) return;
  const s = S.strokes[idx];
  if (s.id) sendWS(JSON.stringify({type: 'erase', ids: [s.id]}));
  S.strokes.splice(idx, 1);
  S.selectedId = null;
  S.transforming = null;
  saveState();
  redraw();
}

export function resize() {
  const w = window.innerWidth - 48;
  const h = window.innerHeight - 44 - 52;
  canvas.width = w; canvas.height = h;
  cursorCanvas.width = w; cursorCanvas.height = h;
  redraw();
}
