import { get_bounds, UndoRedo, epsilon, segments, grid_size, throttle_draw, throttle_cursor, cap_floats, incr_throttle, tools, screen_to_canvas, snap as rust_snap } from './pkg/scrib.js';

export const undoRedo = UndoRedo.new();

export const EPSILON = epsilon();
export const SEGMENTS = segments();
export const GRID = grid_size();
export const THROTTLE_DRAW = throttle_draw();
export const THROTTLE_CURSOR = throttle_cursor();
export const TOOLS = tools();
export const CAP_FLOATS = cap_floats();
export const INCR_THROTTLE = incr_throttle();

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
  selectedId: null,
  transforming: null,

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
  undoRedo.save(JSON.stringify(S.strokes));
}

function screenToCanvas(sx, sy) {
  const c = screen_to_canvas(sx, sy, S.camX, S.camY, S.camZoom);
  return { x: c[0], y: c[1] };
}

export function getPos(e) {
  const r = canvas.getBoundingClientRect();
  const c = screenToCanvas(e.clientX - r.left, e.clientY - r.top);
  return { x: c.x, y: c.y, p: e.pressure ?? 0.5 };
}

export function snap(val) { return rust_snap(val, S.showGrid); }

export function getBounds(s) {
  const b = get_bounds(s.type, s.x||0, s.y||0, s.x2||0, s.y2||0, s.size, s.data||[]);
  return { x1: b[0], y1: b[1], x2: b[2], y2: b[3] };
}

export function getHandles(b) {
  const cx = (b.x1 + b.x2) / 2, cy = (b.y1 + b.y2) / 2;
  return [
    { id: 'nw', x: b.x1, y: b.y1 }, { id: 'n', x: cx, y: b.y1 }, { id: 'ne', x: b.x2, y: b.y1 },
    { id: 'w', x: b.x1, y: cy },                                 { id: 'e', x: b.x2, y: cy },
    { id: 'sw', x: b.x1, y: b.y2 }, { id: 's', x: cx, y: b.y2 }, { id: 'se', x: b.x2, y: b.y2 },
  ];
}
