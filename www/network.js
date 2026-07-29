import { shape_mesh } from './pkg/scrib.js';
import { S, THROTTLE_CURSOR, saveState } from './state.js';
import { redraw } from './render.js';

export function onWSInit(msg) {
  S.myId = msg.id;
  S.myColor = msg.color;
  for (const u of msg.users)
    if (u.id !== S.myId) S.remoteCursors[u.id] = {color: u.color, x: -999, y: -999, rx: -999, ry: -999, tx: -999, ty: -999};
  if (msg.strokes) {
    for (const entry of msg.strokes) {
      const s = entry.stroke;
      s.userId = entry.user_id;
      if (!s.id) s.id = 's_' + Date.now().toString(36) + Math.random().toString(36).slice(2, 8);
      if (s.type !== 'path' && s.type !== 'dot' && !s.mesh) {
        s.mesh = Array.from(shape_mesh(s.type, s.x1, s.y1, s.x2, s.y2, s.size, 32));
      }
      S.strokes.push(s);
    }
  }
  updateUserCount();
}

export function onWSJoin(msg) {
  S.remoteCursors[msg.id] = {color: msg.color, x: -999, y: -999, rx: -999, ry: -999, tx: -999, ty: -999};
  updateUserCount();
}

export function onWSLeave(msg) {
  delete S.remoteCursors[msg.id];
  for (let i = S.strokes.length - 1; i >= 0; i--)
    if (S.strokes[i].userId === msg.id) S.strokes.splice(i, 1);
  updateUserCount();
  redraw();
}

export function onWSStrokeEnd(msg) {
  if (!msg.data) return;
  delete S.liveStrokes[msg.data.id];
  delete S.liveShapes[msg.data.id];
  const s = msg.data.stroke || msg.data;
  s.userId = msg.id;
  if (!s.id) s.id = msg.data.id;
  if (s.type !== 'path' && s.type !== 'dot' && !s.mesh) {
    s.mesh = Array.from(shape_mesh(s.type, s.x1, s.y1, s.x2, s.y2, s.size, 32));
  }
  S.strokes.push(s);
  redraw();
}

export function onWSStrokeStart(msg) {
  if (!msg.data) return;
  S.liveStrokes[msg.data.id] = { mesh: null, color: msg.data.color, size: msg.data.size };
  redraw();
}

export function onWSStrokeUpdate(msg) {
  if (!msg.data || !S.liveStrokes[msg.data.id]) return;
  S.liveStrokes[msg.data.id].mesh = msg.data.mesh;
  redraw();
}

export function onWSShapeUpdate(msg) {
  if (!msg.data) return;
  const shape = msg.data.shape;
  if (!shape || shape.type === 'path' || shape.type === 'dot') return;
  let s = S.liveShapes[msg.data.id];
  const first = !s;
  if (first) {
    s = { type: shape.type, color: shape.color, size: shape.size };
    S.liveShapes[msg.data.id] = s;
  }
  s.tx1 = shape.x1; s.ty1 = shape.y1; s.tx2 = shape.x2; s.ty2 = shape.y2;
  s.color = shape.color;
  s.size = shape.size;
  if (first) {
    s.rx1 = shape.x1; s.ry1 = shape.y1; s.rx2 = shape.x2; s.ry2 = shape.y2;
    s.mesh = Array.from(shape_mesh(s.type, s.rx1, s.ry1, s.rx2, s.ry2, s.size, 32));
    redraw();
  }
}

export function onWSErase(msg) {
  if (!msg.ids || msg.ids.length === 0) return;
  saveState();
  for (let i = 0; i < msg.ids.length; i++) {
    const id = msg.ids[i];
    const ownerId = msg.owners ? msg.owners[i] : null;
    let found = false;
    for (let j = S.strokes.length - 1; j >= 0; j--) {
      if (S.strokes[j].id === id) { S.strokes.splice(j, 1); found = true; break; }
    }
    if (!found && ownerId !== null) {
      for (let j = S.strokes.length - 1; j >= 0; j--) {
        if (S.strokes[j].userId === ownerId) { S.strokes.splice(j, 1); break; }
      }
    }
  }
  redraw();
}

export function onWSCursor(msg) {
  if (!S.remoteCursors[msg.id]) return;
  const c = S.remoteCursors[msg.id];
  if (c.tx === -999) { c.rx = msg.x; c.ry = msg.y; }
  c.tx = msg.x;
  c.ty = msg.y;
}

export function initWS() {
  S.roomId = location.hash.slice(1) || Math.random().toString(36).slice(2, 7);
  location.hash = S.roomId;
  const url = `ws://${location.hostname}:9876`;
  document.getElementById('bottomInfo').textContent = `Room: ${S.roomId} · connecting...`;
  S.ws = new WebSocket(url);
  S.ws.onopen = () => S.ws.send(JSON.stringify({type: 'join', room: S.roomId}));
  S.ws.onmessage = (e) => {
    const msg = JSON.parse(e.data);
    switch (msg.type) {
      case 'init': onWSInit(msg); break;
      case 'join': onWSJoin(msg); break;
      case 'leave': onWSLeave(msg); break;
      case 'stroke-end': case 'stroke': onWSStrokeEnd(msg); break;
      case 'stroke-start': onWSStrokeStart(msg); break;
      case 'stroke-update': onWSStrokeUpdate(msg); break;
      case 'shape-update': onWSShapeUpdate(msg); break;
      case 'cursor': onWSCursor(msg); break;
      case 'erase': onWSErase(msg); break;
    }
  };
  S.ws.onclose = () => {
    document.getElementById('bottomInfo').textContent = `Room: ${S.roomId} · disconnected`;
    S.ws = null;
  };
}

function updateUserCount() {
  document.getElementById('bottomInfo').textContent = `Room: ${S.roomId} · ${Object.keys(S.remoteCursors).length + 1} online`;
}

export function sendCursor(x, y) {
  const now = Date.now();
  if (now - S.cursorThrottle < THROTTLE_CURSOR) return;
  S.cursorThrottle = now;
  if (S.ws?.readyState === WebSocket.OPEN) S.ws.send(JSON.stringify({type: 'cursor', x, y}));
}
