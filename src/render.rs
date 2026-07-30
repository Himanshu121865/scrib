use js_sys::{Array, Number, Reflect};
use std::f64::consts::PI;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

use crate::js_helpers::{get, get_f64, get_keys, get_str, set};

pub fn draw_mesh(ctx: &CanvasRenderingContext2d, data: &[f64], color: &str) {
    ctx.set_fill_style_str(color);
    ctx.begin_path();
    let mut i = 0;
    while i + 5 < data.len() {
        ctx.move_to(data[i], data[i + 1]);
        ctx.line_to(data[i + 2], data[i + 3]);
        ctx.line_to(data[i + 4], data[i + 5]);
        ctx.close_path();
        i += 6;
    }
    ctx.fill();
}

pub fn draw_dot(
    ctx: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    color: &str,
    size: f64,
    pressure: f64,
) {
    ctx.set_fill_style_str(color);
    let r = (size / 2.0) * (0.3 + 0.7 * pressure);
    ctx.begin_path();
    let _ = ctx.arc(x, y, r, 0.0, PI * 2.0);
    ctx.fill();
}

pub fn draw_rect_shape(
    ctx: &CanvasRenderingContext2d,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: &str,
    width: f64,
) {
    ctx.set_stroke_style_str(color);
    ctx.set_line_width(width);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    ctx.stroke_rect(x1.min(x2), y1.min(y2), (x2 - x1).abs(), (y2 - y1).abs());
}

pub fn draw_circle_shape(
    ctx: &CanvasRenderingContext2d,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: &str,
    width: f64,
) {
    ctx.set_stroke_style_str(color);
    ctx.set_line_width(width);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    let cx = (x1 + x2) / 2.0;
    let cy = (y1 + y2) / 2.0;
    let r = (x2 - x1).hypot(y2 - y1) / 2.0;
    ctx.begin_path();
    let _ = ctx.arc(cx, cy, r, 0.0, PI * 2.0);
    ctx.stroke();
}

pub fn draw_line_shape(
    ctx: &CanvasRenderingContext2d,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: &str,
    width: f64,
) {
    ctx.set_stroke_style_str(color);
    ctx.set_line_width(width);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    ctx.begin_path();
    ctx.move_to(x1, y1);
    ctx.line_to(x2, y2);
    ctx.stroke();
}

pub fn draw_arrow_shape(
    ctx: &CanvasRenderingContext2d,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: &str,
    width: f64,
) {
    draw_line_shape(ctx, x1, y1, x2, y2, color, width);
    let dx = x2 - x1;
    let dy = y2 - y1;
    if dx.hypot(dy) < 1.0 {
        return;
    }
    let angle = dy.atan2(dx);
    let hl = (width * 3.0).max(8.0);
    let ha = PI / 6.0;
    ctx.begin_path();
    ctx.move_to(x2, y2);
    ctx.line_to(x2 - hl * (angle - ha).cos(), y2 - hl * (angle - ha).sin());
    ctx.move_to(x2, y2);
    ctx.line_to(x2 - hl * (angle + ha).cos(), y2 - hl * (angle + ha).sin());
    ctx.stroke();
}

#[allow(clippy::too_many_arguments)]
pub fn draw_shape_canvas(
    ctx: &CanvasRenderingContext2d,
    kind: &str,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: &str,
    width: f64,
) {
    match kind {
        "rect" => draw_rect_shape(ctx, x1, y1, x2, y2, color, width),
        "circle" => draw_circle_shape(ctx, x1, y1, x2, y2, color, width),
        "line" => draw_line_shape(ctx, x1, y1, x2, y2, color, width),
        "arrow" => draw_arrow_shape(ctx, x1, y1, x2, y2, color, width),
        _ => {}
    }
}

pub fn draw_cursor_marker(ctx: &CanvasRenderingContext2d, x: f64, y: f64, color: &str) {
    ctx.begin_path();
    let _ = ctx.arc(x, y, 5.0, 0.0, PI * 2.0);
    ctx.set_fill_style_str(color);
    ctx.fill();
    ctx.set_stroke_style_str("rgba(0,0,0,0.25)");
    ctx.set_line_width(1.5);
    ctx.stroke();
}

pub fn draw_selection_box(
    ctx: &CanvasRenderingContext2d,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    zoom: f64,
) {
    let pad = 4.0 / zoom;
    let lw = 1.5 / zoom;
    ctx.set_stroke_style_str("#e86a20");
    ctx.set_line_width(lw);
    ctx.stroke_rect(x1 - pad, y1 - pad, x2 - x1 + pad * 2.0, y2 - y1 + pad * 2.0);
    ctx.set_fill_style_str("#e86a20");
    let hs = 4.0 / zoom;
    let cx = (x1 + x2) / 2.0;
    let cy = (y1 + y2) / 2.0;
    let handles = [
        (x1, y1),
        (cx, y1),
        (x2, y1),
        (x1, cy),
        (x2, cy),
        (x1, y2),
        (cx, y2),
        (x2, y2),
    ];
    for &(hx, hy) in &handles {
        ctx.fill_rect(hx - hs, hy - hs, hs * 2.0, hs * 2.0);
    }
}

pub struct GridParams<'a> {
    pub ctx: &'a CanvasRenderingContext2d,
    pub show_grid: bool,
    pub bg_color: &'a str,
    pub cam_x: f64,
    pub cam_y: f64,
    pub cam_zoom: f64,
    pub canvas_w: f64,
    pub canvas_h: f64,
    pub grid: f64,
}

pub fn draw_grid(p: &GridParams) {
    if !p.show_grid {
        return;
    }
    let col = if p.bg_color == "#ffffff" {
        "#e0e0e0"
    } else {
        "#333333"
    };
    let l = -p.cam_x / p.cam_zoom;
    let t = -p.cam_y / p.cam_zoom;
    let r = l + p.canvas_w / p.cam_zoom;
    let b = t + p.canvas_h / p.cam_zoom;
    p.ctx.set_stroke_style_str(col);
    p.ctx.set_line_width(1.0);
    p.ctx.begin_path();
    let mut x = (l / p.grid).floor() * p.grid;
    while x <= r {
        let sx = x * p.cam_zoom + p.cam_x;
        p.ctx.move_to(sx, 0.0);
        p.ctx.line_to(sx, p.canvas_h);
        x += p.grid;
    }
    let mut y = (t / p.grid).floor() * p.grid;
    while y <= b {
        let sy = y * p.cam_zoom + p.cam_y;
        p.ctx.move_to(0.0, sy);
        p.ctx.line_to(p.canvas_w, sy);
        y += p.grid;
    }
    p.ctx.stroke();
}

fn lerp_cursors(remote_cursors: &JsValue) -> bool {
    let mut dirty = false;
    let keys = get_keys(remote_cursors);
    let len = keys.length();
    for i in 0..len {
        let key = keys.get(i);
        if !key.is_string() {
            continue;
        }
        let cursor = match Reflect::get(remote_cursors, &key) {
            Ok(c) => c,
            _ => continue,
        };
        if !cursor.is_object() {
            continue;
        }

        let tx_val = get(&cursor, "tx");
        if tx_val.as_f64() == Some(-999.0) || tx_val.is_undefined() {
            continue;
        }
        let tx = get_f64(&cursor, "tx");
        let ty = get_f64(&cursor, "ty");
        let rx = get_f64(&cursor, "rx");
        let ry = get_f64(&cursor, "ry");

        let dx = tx - rx;
        let dy = ty - ry;

        if dx.abs() < 0.5 && dy.abs() < 0.5 {
            if (rx - tx).abs() > 1e-15 || (ry - ty).abs() > 1e-15 {
                set(&cursor, "rx", &Number::from(tx));
                set(&cursor, "ry", &Number::from(ty));
                dirty = true;
            }
        } else {
            set(&cursor, "rx", &Number::from(rx + dx * 0.25));
            set(&cursor, "ry", &Number::from(ry + dy * 0.25));
            dirty = true;
        }
    }
    dirty
}

fn lerp_shapes(live_shapes: &JsValue) -> bool {
    let mut dirty = false;
    let keys = get_keys(live_shapes);
    let len = keys.length();
    for i in 0..len {
        let key = keys.get(i);
        if !key.is_string() {
            continue;
        }
        let shape = match Reflect::get(live_shapes, &key) {
            Ok(s) => s,
            _ => continue,
        };
        if !shape.is_object() {
            continue;
        }

        let tx1_val = get(&shape, "tx1");
        if tx1_val.is_undefined() {
            continue;
        }
        let tx1 = get_f64(&shape, "tx1");
        let ty1 = get_f64(&shape, "ty1");
        let tx2 = get_f64(&shape, "tx2");
        let ty2 = get_f64(&shape, "ty2");
        let rx1 = get_f64(&shape, "rx1");
        let ry1 = get_f64(&shape, "ry1");
        let rx2 = get_f64(&shape, "rx2");
        let ry2 = get_f64(&shape, "ry2");

        let dx1 = tx1 - rx1;
        let dy1 = ty1 - ry1;
        let dx2 = tx2 - rx2;
        let dy2 = ty2 - ry2;

        let snapped = dx1.abs() < 0.5 && dy1.abs() < 0.5 && dx2.abs() < 0.5 && dy2.abs() < 0.5;

        let (nr1, nry1, nrx2, nry2) = if snapped {
            (tx1, ty1, tx2, ty2)
        } else {
            (
                rx1 + dx1 * 0.25,
                ry1 + dy1 * 0.25,
                rx2 + dx2 * 0.25,
                ry2 + dy2 * 0.25,
            )
        };

        let changed = (nr1 - rx1).abs() > 1e-15
            || (nry1 - ry1).abs() > 1e-15
            || (nrx2 - rx2).abs() > 1e-15
            || (nry2 - ry2).abs() > 1e-15;

        if !changed && !snapped {
            continue;
        }

        set(&shape, "rx1", &Number::from(nr1));
        set(&shape, "ry1", &Number::from(nry1));
        set(&shape, "rx2", &Number::from(nrx2));
        set(&shape, "ry2", &Number::from(nry2));

        let kind = get_str(&shape, "type");
        let sz = get_f64(&shape, "size");
        let mesh_data = crate::wasm::generate_shape_mesh(&kind, nr1, nry1, nrx2, nry2, sz);
        let arr = Array::new();
        for v in mesh_data {
            arr.push(&Number::from(v));
        }
        set(&shape, "mesh", &arr);
        dirty = true;
    }
    dirty
}

pub fn tick_animation(remote_cursors: &JsValue, live_shapes: &JsValue) -> u8 {
    let d1 = lerp_cursors(remote_cursors);
    let d2 = lerp_shapes(live_shapes);
    (if d1 { 1 } else { 0 }) | (if d2 { 2 } else { 0 })
}
