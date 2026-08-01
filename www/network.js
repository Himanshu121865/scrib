import { WsClient, ws_init, ws_join, ws_leave, ws_stroke_end, ws_stroke_start, ws_stroke_update, ws_shape_update, ws_cursor, ws_erase } from './pkg/scrib.js';
import { S, THROTTLE_CURSOR, saveState } from './state.js';
import { redraw } from './render.js';

let client = null;
export function initNetworkClient() {
  client = WsClient.new();
}

export function sendWS(json) {
  client.send(json);
}

function onWSMessage(text) {
  const msg = JSON.parse(text);
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
}

function onWSStatus(text) {
  document.getElementById('bottomInfo').textContent = text;
  if (text === 'disconnected') updateUserCount();
}

function onWSInit(msg) {
  ws_init(msg, S.remoteCursors, S.strokes);
  S.myId = msg.id;
  S.myColor = msg.color;
  updateUserCount();
  redraw();
}

function onWSJoin(msg) {
  ws_join(msg, S.remoteCursors);
  updateUserCount();
}

function onWSLeave(msg) {
  ws_leave(msg, S.remoteCursors, S.strokes);
  S.selectedId = null;
  S.transforming = null;
  updateUserCount();
  redraw();
}

function onWSStrokeEnd(msg) {
  ws_stroke_end(msg, S.liveStrokes, S.liveShapes, S.strokes);
  redraw();
}

function onWSStrokeStart(msg) {
  ws_stroke_start(msg, S.liveStrokes);
  redraw();
}

function onWSStrokeUpdate(msg) {
  ws_stroke_update(msg, S.liveStrokes);
  redraw();
}

function onWSShapeUpdate(msg) {
  const first = ws_shape_update(msg, S.liveShapes);
  if (first) redraw();
}

function onWSErase(msg) {
  saveState();
  ws_erase(msg, S.strokes);
  S.selectedId = null;
  S.transforming = null;
  redraw();
}

function onWSCursor(msg) {
  ws_cursor(msg, S.remoteCursors);
}

export function initWS() {
  const url = `ws://${location.hostname}:9876`;
  document.getElementById('bottomInfo').textContent = 'connecting...';
  client.connect(url, onWSMessage, onWSStatus);
}

function updateUserCount() {
  document.getElementById('bottomInfo').textContent = `${Object.keys(S.remoteCursors).length + 1} online`;
}

export function sendCursor(x, y) {
  const now = Date.now();
  if (now - S.cursorThrottle < THROTTLE_CURSOR) return;
  S.cursorThrottle = now;
  sendWS(JSON.stringify({type: 'cursor', x, y}));
}
