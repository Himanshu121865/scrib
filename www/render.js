import { shape_mesh } from './pkg/scrib.js';
import { S, canvas, cursorCanvas, GRID, ctx, cursorCtx } from './state.js';

function dot(x, y, color, size, pressure) {
  ctx.fillStyle = color;
  const r = (size / 2) * (0.3 + 0.7 * pressure);
  ctx.beginPath();
  ctx.arc(x, y, r, 0, Math.PI * 2);
  ctx.fill();
}

function drawMesh(data, color) {
  ctx.fillStyle = color;
  ctx.beginPath();
  for (let i = 0; i < data.length; i += 6) {
    ctx.moveTo(data[i], data[i + 1]);
    ctx.lineTo(data[i + 2], data[i + 3]);
    ctx.lineTo(data[i + 4], data[i + 5]);
    ctx.closePath();
  }
  ctx.fill();
}

function drawShape(s) {
  if (s.mesh) { drawMesh(s.mesh, s.color); return; }
  ctx.strokeStyle = s.color;
  ctx.lineWidth = s.size;
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';
  ctx.beginPath();
  if (s.type === 'rect') {
    ctx.rect(Math.min(s.x1, s.x2), Math.min(s.y1, s.y2), Math.abs(s.x2 - s.x1), Math.abs(s.y2 - s.y1));
  } else if (s.type === 'circle') {
    const cx = (s.x1 + s.x2) / 2, cy = (s.y1 + s.y2) / 2;
    ctx.arc(cx, cy, Math.hypot(s.x2 - s.x1, s.y2 - s.y1) / 2, 0, Math.PI * 2);
  } else if (s.type === 'line' || s.type === 'arrow') {
    ctx.moveTo(s.x1, s.y1); ctx.lineTo(s.x2, s.y2);
  }
  ctx.stroke();
  if (s.type === 'arrow' && Math.hypot(s.x2 - s.x1, s.y2 - s.y1) >= 1) {
    const a = Math.atan2(s.y2 - s.y1, s.x2 - s.x1), hl = s.size * 3, ha = Math.PI / 6;
    ctx.beginPath();
    ctx.moveTo(s.x2, s.y2);
    ctx.lineTo(s.x2 - hl * Math.cos(a - ha), s.y2 - hl * Math.sin(a - ha));
    ctx.moveTo(s.x2, s.y2);
    ctx.lineTo(s.x2 - hl * Math.cos(a + ha), s.y2 - hl * Math.sin(a + ha));
    ctx.stroke();
  }
}

function drawGrid() {
  if (!S.showGrid) return;
  const col = S.bgColor === '#ffffff' ? '#e0e0e0' : '#333333';
  const l = -S.camX / S.camZoom, t = -S.camY / S.camZoom;
  const r = l + canvas.width / S.camZoom, b = t + canvas.height / S.camZoom;
  ctx.strokeStyle = col;
  ctx.lineWidth = 1;
  ctx.beginPath();
  for (let x = Math.floor(l / GRID) * GRID; x <= r; x += GRID) {
    const sx = x * S.camZoom + S.camX;
    ctx.moveTo(sx, 0); ctx.lineTo(sx, canvas.height);
  }
  for (let y = Math.floor(t / GRID) * GRID; y <= b; y += GRID) {
    const sy = y * S.camZoom + S.camY;
    ctx.moveTo(0, sy); ctx.lineTo(canvas.width, sy);
  }
  ctx.stroke();
}

function drawCursor(c, cxt) {
  const g = cxt || ctx;
  g.beginPath();
  g.arc(c.rx, c.ry, 5, 0, Math.PI * 2);
  g.fillStyle = c.color;
  g.fill();
  g.strokeStyle = 'rgba(0,0,0,0.25)';
  g.lineWidth = 1.5;
  g.stroke();
}

export function redraw() {
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.fillStyle = S.bgColor;
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  drawGrid();
  ctx.setTransform(S.camZoom, 0, 0, S.camZoom, S.camX, S.camY);

  for (let i = 0; i < S.strokes.length; i++) {
    const s = S.strokes[i];
    ctx.globalAlpha = S.toErase.has(i) ? 0.25 : 1;
    if (s.type === 'dot') dot(s.x, s.y, s.color, s.size, s.pressure);
    else if (s.type === 'path') drawMesh(s.mesh, s.color);
    else drawShape(s);
  }
  ctx.globalAlpha = 1;

  for (const id in S.liveStrokes) { const ls = S.liveStrokes[id]; if (ls.mesh) drawMesh(ls.mesh, ls.color); }
  for (const id in S.liveShapes) drawShape(S.liveShapes[id]);

  if (S.shaping && S.shapeStart) {
    const previewMesh = shape_mesh(S.currentTool, S.shapeStart.x, S.shapeStart.y, S.shapeStart.x2, S.shapeStart.y2, S.currentSize, 32);
    drawMesh(previewMesh, S.currentColor);
  }

  if (S.currentTool === 'draw' && S.currentRaw && S.currentRaw.length >= 3) {
    if (S.currentRaw.length === 3) dot(S.currentRaw[0], S.currentRaw[1], S.currentColor, S.currentSize, S.currentRaw[2]);
    else if (S.cachedMesh) drawMesh(S.cachedMesh, S.currentColor);
  }

  redrawCursors();
}

export function redrawCursors() {
  cursorCtx.setTransform(1, 0, 0, 1, 0, 0);
  cursorCtx.clearRect(0, 0, cursorCanvas.width, cursorCanvas.height);
  cursorCtx.setTransform(S.camZoom, 0, 0, S.camZoom, S.camX, S.camY);
  for (const id in S.remoteCursors) {
    const c = S.remoteCursors[id];
    if (c.rx > -900) drawCursor(c, cursorCtx);
  }
  cursorCtx.setTransform(1, 0, 0, 1, 0, 0);
}

export function tickAnimation() {
  let dirtyCursors = false;
  let dirtyShapes = false;
  for (const id in S.remoteCursors) {
    const c = S.remoteCursors[id];
    if (c.tx === -999) continue;
    const dx = c.tx - c.rx;
    const dy = c.ty - c.ry;
    if (Math.abs(dx) < 0.5 && Math.abs(dy) < 0.5) {
      if (c.rx !== c.tx) { c.rx = c.tx; c.ry = c.ty; dirtyCursors = true; }
    } else {
      c.rx += dx * 0.25;
      c.ry += dy * 0.25;
      dirtyCursors = true;
    }
  }
  for (const id in S.liveShapes) {
    const s = S.liveShapes[id];
    if (s.tx1 === undefined) continue;
    const dx1 = s.tx1 - s.rx1, dy1 = s.ty1 - s.ry1;
    const dx2 = s.tx2 - s.rx2, dy2 = s.ty2 - s.ry2;
    if (Math.abs(dx1) < 0.5 && Math.abs(dy1) < 0.5 && Math.abs(dx2) < 0.5 && Math.abs(dy2) < 0.5) {
      if (s.rx1 !== s.tx1 || s.ry1 !== s.ty1 || s.rx2 !== s.tx2 || s.ry2 !== s.ty2) {
        s.rx1 = s.tx1; s.ry1 = s.ty1; s.rx2 = s.tx2; s.ry2 = s.ty2;
        s.mesh = Array.from(shape_mesh(s.type, s.rx1, s.ry1, s.rx2, s.ry2, s.size, 32));
        dirtyShapes = true;
      }
    } else {
      s.rx1 += dx1 * 0.25; s.ry1 += dy1 * 0.25;
      s.rx2 += dx2 * 0.25; s.ry2 += dy2 * 0.25;
      s.mesh = Array.from(shape_mesh(s.type, s.rx1, s.ry1, s.rx2, s.ry2, s.size, 32));
      dirtyShapes = true;
    }
  }
  if (dirtyShapes) redraw();
  else if (dirtyCursors) redrawCursors();
  requestAnimationFrame(tickAnimation);
}
