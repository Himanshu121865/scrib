export const EPSILON = 0.5;
export const SEGMENTS = 6;
export const MAX_UNDO = 100;
export const GRID = 30;
export const THROTTLE_DRAW = 80;
export const THROTTLE_CURSOR = 40;
export const TOOLS = ['draw', 'eraser', 'rect', 'circle', 'line', 'arrow'];
export const CAP_FLOATS = 8 * 6;
export const INCR_THROTTLE = 30;

export const canvas = document.getElementById('canvas');
export const ctx = canvas.getContext('2d');
export const cursorCanvas = document.getElementById('cursorCanvas');
export const cursorCtx = cursorCanvas.getContext('2d');

export const S = {
  strokes: [],
  undoStack: [],
  redoStack: [],
  currentRaw: null,
  drawing: false,
  shaping: false,
  shapeStart: null,
  erasing: false,
  eraserPath: [],
  toErase: new Set(),
  currentColor: '#eee',
  currentSize: undefined,
  currentTool: 'draw',

  camX: 0, camY: 0, camZoom: 1,
  panning: false, panBtn: -1,
  panStartX: 0, panStartY: 0,
  panStartCamX: 0, panStartCamY: 0,
  spaceDown: false,
  showGrid: false,
  bgColor: '#1a1a1a',
  touchPoints: new Map(),
  pinchStartDist: 0, pinchStartZoom: 1,
  pinchCenterX: 0, pinchCenterY: 0,
  pinchCamX: 0, pinchCamY: 0,

  ws: null,
  myId: -1,
  myColor: '#e86a20',
  roomId: '',
  remoteCursors: {},
  cursorThrottle: 0,
  currentStrokeId: null,
  currentShapeId: null,
  lastStrokeSendTime: 0,
  lastIncrTime: 0,
  cachedMesh: null,
  liveStrokes: {},
  liveShapes: {},
  cachedCenterline: null,
};

export function saveState() {
  S.undoStack.push(JSON.parse(JSON.stringify(S.strokes)));
  if (S.undoStack.length > MAX_UNDO) S.undoStack.shift();
  S.redoStack.length = 0;
}

export function screenToCanvas(sx, sy) {
  return { x: (sx - S.camX) / S.camZoom, y: (sy - S.camY) / S.camZoom };
}

export function getPos(e) {
  const r = canvas.getBoundingClientRect();
  const c = screenToCanvas(e.clientX - r.left, e.clientY - r.top);
  return { x: c.x, y: c.y, p: e.pressure || 0.5 };
}

export function distToSegment(px, py, x1, y1, x2, y2) {
  const dx = x2 - x1, dy = y2 - y1, ls = dx * dx + dy * dy;
  if (ls === 0) return Math.hypot(px - x1, py - y1);
  let t = ((px - x1) * dx + (py - y1) * dy) / ls;
  t = Math.max(0, Math.min(1, t));
  return Math.hypot(px - (x1 + t * dx), py - (y1 + t * dy));
}

export function snap(val) { return S.showGrid ? Math.round(val / GRID) * GRID : val; }
