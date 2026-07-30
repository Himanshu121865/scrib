import init from './pkg/scrib.js';
import { S, canvas, TOOLS, saveState, getPos, undoRedo, initWasmConstants } from './state.js';
import { redraw, tickAnimation } from './render.js';
import { cancelErase, resetIncrCache, handleDown, handleMove, handleUp, resize } from './tools.js';
import { initWS, sendCursor } from './network.js';

try {
  await init();
  initWasmConstants();
} catch (e) {
  const msg = location.protocol === 'file:'
    ? 'Open via HTTP server (python3 -m http.server 8080), not file://'
    : 'Failed to load WASM — check browser console';
  document.getElementById('bottomInfo').textContent = msg;
  console.error('WASM init failed:', e);
}

function restoreStrokes(json) {
  if (!json) return;
  S.strokes.length = 0;
  const parsed = JSON.parse(json);
  for (const s of parsed) S.strokes.push(s);
}

function undo() {
  if (!undoRedo.can_undo()) return;
  if (S.drawing) { S.drawing = false; S.currentRaw = null; }
  if (S.shaping) { S.shaping = false; S.shapeStart = null; }
  if (S.erasing) cancelErase();
  resetIncrCache();
  S.selectedId = null; S.transforming = null;
  const prev = undoRedo.undo(JSON.stringify(S.strokes));
  restoreStrokes(prev);
  redraw();
}

function redo() {
  if (!undoRedo.can_redo()) return;
  if (S.drawing) { S.drawing = false; S.currentRaw = null; }
  if (S.shaping) { S.shaping = false; S.shapeStart = null; }
  if (S.erasing) cancelErase();
  resetIncrCache();
  S.selectedId = null; S.transforming = null;
  const next = undoRedo.redo(JSON.stringify(S.strokes));
  restoreStrokes(next);
  redraw();
}

function genId() {
  return Date.now().toString(36) + Math.random().toString(36).slice(2, 8);
}

function copySelected() {
  if (S.selectedId === null) return;
  const sel = S.strokes.find(s => s.id === S.selectedId);
  if (!sel) return;
  S.clipboard = JSON.parse(JSON.stringify(sel));
}

function pasteClipboard() {
  if (!S.clipboard) return;
  S.currentTool = 'select';
  for (const id of TOOLS)
    document.getElementById('tool' + id)?.classList.toggle('active', id === 'select');
  canvas.style.cursor = 'default';

  const copy = JSON.parse(JSON.stringify(S.clipboard));
  copy.id = genId();
  copy.userId = '';
  const off = S.showGrid ? 30 : 20;
  if (copy.type === 'dot') {
    copy.x += off;
    copy.y += off;
  } else if (copy.type === 'path') {
    for (let i = 0; i + 2 < copy.data.length; i += 3) {
      copy.data[i] += off;
      copy.data[i + 1] += off;
    }
  } else {
    copy.x1 += off; copy.y1 += off;
    copy.x2 += off; copy.y2 += off;
  }
  saveState();
  S.strokes.push(copy);
  S.selectedId = copy.id;
  S.transforming = null;
  redraw();
}

function setTool(tool) {
  if (S.shaping) { S.shaping = false; S.shapeStart = null; redraw(); }
  if (S.drawing) { S.drawing = false; S.currentRaw = null; }
  if (S.erasing) cancelErase();
  resetIncrCache();
  S.selectedId = null;
  S.transforming = null;
  S.currentTool = tool;
  for (const id of TOOLS)
    document.getElementById('tool' + id)?.classList.toggle('active', id === tool);
  canvas.style.cursor = tool === 'eraser' ? 'pointer' : tool === 'select' ? 'default' : 'crosshair';
  redraw();
}


initWS();
tickAnimation();


const sizeSlider = document.getElementById('sizeSlider');
const sizeLabel = document.getElementById('sizeLabel');
function updateSize() {
  S.currentSize = parseFloat(sizeSlider.value);
  sizeLabel.textContent = S.currentSize.toFixed(1);
  const p = document.getElementById('sizePreview');
  const s = Math.max(2, Math.min(20, S.currentSize * 2));
  p.style.width = s + 'px'; p.style.height = s + 'px'; p.style.background = S.currentColor;
}
sizeSlider.addEventListener('input', updateSize);
updateSize();


document.querySelectorAll('.swatch').forEach(el => {
  el.addEventListener('click', () => {
    document.querySelectorAll('.swatch').forEach(s => s.classList.remove('active'));
    el.classList.add('active');
    S.currentColor = el.dataset.color;
    document.getElementById('customColorBtn').style.background = '#2a2a2a';
    document.getElementById('customColorBtn').classList.remove('active');
    updateSize();
  });
});
document.querySelector('.swatch[data-color="#eee"]').classList.add('active');
document.getElementById('customColorBtn').addEventListener('click', () => document.getElementById('customColor').click());
document.getElementById('customColor').addEventListener('input', (e) => {
  S.currentColor = e.target.value;
  document.querySelectorAll('.swatch').forEach(s => s.classList.remove('active'));
  const btn = document.getElementById('customColorBtn');
  btn.style.background = S.currentColor;
  btn.classList.add('active');
  updateSize();
});


for (const t of TOOLS)
  document.getElementById('tool' + t)?.addEventListener('click', () => setTool(t));


document.getElementById('zoomIn').addEventListener('click', () => {
  const step = Math.round(S.camZoom / 0.25);
  const newZoom = Math.max(0.25, Math.min(10, (step + 1) * 0.25));
  const cx = (canvas.width / 2 - S.camX) / S.camZoom;
  const cy = (canvas.height / 2 - S.camY) / S.camZoom;
  S.camX = canvas.width / 2 - cx * newZoom;
  S.camY = canvas.height / 2 - cy * newZoom;
  S.camZoom = newZoom;
  document.getElementById('zoomLabel').textContent = Math.round(S.camZoom * 100) + '%';
  redraw();
});
document.getElementById('zoomOut').addEventListener('click', () => {
  const step = Math.round(S.camZoom / 0.25);
  const newZoom = Math.max(0.25, Math.min(10, (step - 1) * 0.25));
  const cx = (canvas.width / 2 - S.camX) / S.camZoom;
  const cy = (canvas.height / 2 - S.camY) / S.camZoom;
  S.camX = canvas.width / 2 - cx * newZoom;
  S.camY = canvas.height / 2 - cy * newZoom;
  S.camZoom = newZoom;
  document.getElementById('zoomLabel').textContent = Math.round(S.camZoom * 100) + '%';
  redraw();
});
document.getElementById('zoomReset').addEventListener('click', () => {
  S.camX = 0; S.camY = 0; S.camZoom = 1;
  document.getElementById('zoomLabel').textContent = '100%';
  redraw();
});


document.getElementById('gridToggle').addEventListener('click', () => {
  S.showGrid = !S.showGrid;
  document.getElementById('gridToggle').classList.toggle('tog-on', S.showGrid);
  redraw();
});


document.getElementById('bgToggle').addEventListener('click', () => {
  const isDark = S.bgColor !== '#1a1a1a';
  S.bgColor = isDark ? '#1a1a1a' : '#ffffff';
  document.getElementById('bgToggle').classList.toggle('tog-on', isDark);
  document.getElementById('bgIcon').innerHTML = isDark
    ? '<path d="M17 10.79A8 8 0 1 1 9.21 3 6 6 0 0 0 17 10.79z"/>'
    : '<circle cx="10" cy="10" r="4"/><line x1="10" y1="1" x2="10" y2="3"/><line x1="10" y1="17" x2="10" y2="19"/><line x1="3.05" y1="3.05" x2="4.46" y2="4.46"/><line x1="15.54" y1="15.54" x2="16.95" y2="16.95"/><line x1="1" y1="10" x2="3" y2="10"/><line x1="17" y1="10" x2="19" y2="10"/><line x1="3.05" y1="16.95" x2="4.46" y2="15.54"/><line x1="15.54" y1="4.46" x2="16.95" y2="3.05"/>';
  const newColor = isDark ? '#eee' : '#222';
  S.currentColor = newColor;
  document.querySelectorAll('.swatch').forEach(s => s.classList.remove('active'));
  document.querySelector(`.swatch[data-color="${newColor}"]`).classList.add('active');
  redraw();
});


document.getElementById('undoBtn').addEventListener('click', undo);
document.getElementById('redoBtn').addEventListener('click', redo);


const overlay = document.getElementById('shortcutsOverlay');
document.getElementById('shortcutsBtn').addEventListener('click', () => overlay.classList.toggle('open'));
overlay.addEventListener('click', (e) => { if (e.target === overlay) overlay.classList.remove('open'); });


document.getElementById('clear').addEventListener('click', () => {
  saveState();
  S.strokes.length = 0;
  S.currentRaw = null;
  S.drawing = false;
  S.selectedId = null;
  S.transforming = null;
  resetIncrCache();
  redraw();
});


document.addEventListener('keydown', (e) => {
  if (e.key === ' ') { e.preventDefault(); S.spaceDown = true; }
  if (!e.ctrlKey && !e.metaKey) {
    if (e.key === 's' || e.key === 'S') setTool('select');
    else if (e.key === 'e' || e.key === 'E') setTool('eraser');
    else if (e.key === 'd' || e.key === 'D') setTool('draw');
    else if (e.key === 'r' || e.key === 'R') setTool('rect');
    else if (e.key === 'c' || e.key === 'C') setTool('circle');
    else if (e.key === 'l' || e.key === 'L') setTool('line');
    else if (e.key === 'a' || e.key === 'A') setTool('arrow');
    else if (e.key === 'g' || e.key === 'G') document.getElementById('gridToggle').click();
  }
  if (e.key === 'Escape') { if (overlay.classList.contains('open')) { overlay.classList.remove('open'); return; } if (S.selectedId) { S.selectedId = null; S.transforming = null; redraw(); return; } if (S.currentTool === 'eraser') setTool('draw'); }
  if (e.key === '?' && !e.ctrlKey && !e.metaKey) { overlay.classList.toggle('open'); }
  if ((e.key === 'z' || e.key === 'Z') && (e.ctrlKey || e.metaKey) && e.shiftKey) { redo(); }
  else if (e.key === 'y' && (e.ctrlKey || e.metaKey)) { redo(); }
  else if ((e.key === 'z' || e.key === 'Z') && (e.ctrlKey || e.metaKey)) { undo(); }
  else if ((e.key === 'c' || e.key === 'C') && (e.ctrlKey || e.metaKey)) { e.preventDefault(); copySelected(); }
  else if ((e.key === 'v' || e.key === 'V') && (e.ctrlKey || e.metaKey)) { e.preventDefault(); pasteClipboard(); }
});
document.addEventListener('keyup', (e) => {
  if (e.key === ' ') S.spaceDown = false;
});


canvas.addEventListener('pointerdown', (e) => {
  e.preventDefault();

  if (e.pointerType === 'touch') {
    S.touchPoints.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (S.touchPoints.size === 2) {
      if (S.erasing) cancelErase();
      S.drawing = false;
      S.currentRaw = null;
      const pts = [...S.touchPoints.values()];
      S.pinchStartDist = Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y);
      S.pinchCenterX = (pts[0].x + pts[1].x) / 2;
      S.pinchCenterY = (pts[0].y + pts[1].y) / 2;
      S.pinchStartZoom = S.camZoom;
      S.pinchCamX = S.camX;
      S.pinchCamY = S.camY;
      return;
    }
    if (S.touchPoints.size > 1) return;
  }

  if (e.button === 2) { if (S.erasing) cancelErase(); if (S.currentTool === 'eraser') setTool('draw'); return; }

  if (e.button === 1) {
    S.panning = true; S.panBtn = 1;
    S.panStartX = e.clientX; S.panStartY = e.clientY;
    S.panStartCamX = S.camX; S.panStartCamY = S.camY;
    return;
  }

  if (e.button === 0 && S.spaceDown) {
    S.panning = true; S.panBtn = 0;
    S.panStartX = e.clientX; S.panStartY = e.clientY;
    S.panStartCamX = S.camX; S.panStartCamY = S.camY;
    return;
  }

  if (e.button === 0) {
    const p = getPos(e);
    handleDown(p.x, p.y, p.p);
  }
});

canvas.addEventListener('pointermove', (e) => {
  e.preventDefault();

  if (e.pointerType === 'touch') {
    if (S.touchPoints.has(e.pointerId)) {
      S.touchPoints.set(e.pointerId, { x: e.clientX, y: e.clientY });
    }
    if (S.touchPoints.size === 2) {
      const pts = [...S.touchPoints.values()];
      const dist = Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y);
      const cx = (pts[0].x + pts[1].x) / 2;
      const cy = (pts[0].y + pts[1].y) / 2;
      const dx = cx - S.pinchCenterX;
      const dy = cy - S.pinchCenterY;
      S.camZoom = Math.max(0.1, Math.min(10, S.pinchStartZoom * (dist / S.pinchStartDist)));
      S.camX = S.pinchCamX + dx;
      S.camY = S.pinchCamY + dy;
      document.getElementById('zoomLabel').textContent = Math.round(S.camZoom * 100) + '%';
      redraw();
      return;
    }
    if (S.touchPoints.size > 2) return;
  }

  if (S.panning) {
    S.camX = S.panStartCamX + (e.clientX - S.panStartX);
    S.camY = S.panStartCamY + (e.clientY - S.panStartY);
    redraw();
    return;
  }

  const p = getPos(e);
  sendCursor(p.x, p.y);
  handleMove(p.x, p.y, p.p);
});

canvas.addEventListener('pointerup', (e) => {
  if (e.pointerType === 'touch') {
    S.touchPoints.delete(e.pointerId);
    if (S.touchPoints.size > 0) return;
  }
  if (S.panning) { S.panning = false; return; }
  handleUp();
});

canvas.addEventListener('pointercancel', (e) => {
  if (e.pointerType === 'touch') S.touchPoints.delete(e.pointerId);
  if (S.erasing) cancelErase();
  handleUp();
  S.panning = false;
});

canvas.addEventListener('pointerleave', () => { if (S.erasing) cancelErase(); });

canvas.addEventListener('wheel', (e) => {
  e.preventDefault();
  const rect = canvas.getBoundingClientRect();
  const mx = e.clientX - rect.left;
  const my = e.clientY - rect.top;
  const factor = e.deltaY > 0 ? 0.9 : 1.1;
  const newZoom = Math.max(0.1, Math.min(10, S.camZoom * factor));
  const cx = (mx - S.camX) / S.camZoom;
  const cy = (my - S.camY) / S.camZoom;
  S.camX = mx - cx * newZoom;
  S.camY = my - cy * newZoom;
  S.camZoom = newZoom;
  document.getElementById('zoomLabel').textContent = Math.round(S.camZoom * 100) + '%';
  redraw();
}, { passive: false });

canvas.addEventListener('contextmenu', (e) => {
  e.preventDefault();
  if (S.erasing) cancelErase();
  if (S.currentTool === 'eraser') setTool('draw');
});


window.addEventListener('resize', resize);
resize();
