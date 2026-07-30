import { shape_mesh, render_draw_mesh, render_draw_dot, render_draw_shape_canvas, render_draw_cursor, render_draw_selection, render_draw_grid, tick_animation } from './pkg/scrib.js';
import { S, canvas, cursorCanvas, GRID, ctx, cursorCtx, getBounds } from './state.js';

function dot(x, y, color, size, pressure) {
  render_draw_dot(ctx, x, y, color, size, pressure);
}

function drawMesh(data, color) {
  render_draw_mesh(ctx, data, color);
}

function drawShape(s) {
  if (s.mesh) { drawMesh(s.mesh, s.color); return; }
  render_draw_shape_canvas(ctx, s.type, s.x1||0, s.y1||0, s.x2||0, s.y2||0, s.color, s.size);
}

function drawGrid() {
  render_draw_grid(ctx, S.showGrid, S.bgColor, S.camX, S.camY, S.camZoom, canvas.width, canvas.height, GRID);
}

function drawCursor(c, cxt) {
  render_draw_cursor(cxt || ctx, c.rx, c.ry, c.color);
}

function drawSelection() {
  if (!S.selectedId) return;
  const sel = S.strokes.find(s => s.id === S.selectedId);
  if (!sel) { S.selectedId = null; return; }
  const b = getBounds(sel);
  render_draw_selection(ctx, b.x1, b.y1, b.x2, b.y2, S.camZoom);
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

  drawSelection();
  redrawCursors();
}

function redrawCursors() {
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
  const flags = tick_animation(S.remoteCursors, S.liveShapes);
  if (flags & 2) redraw();
  else if (flags & 1) redrawCursors();
  requestAnimationFrame(tickAnimation);
}
