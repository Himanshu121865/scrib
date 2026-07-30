use js_sys::{Array, Number};
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

use crate::geometry;
use crate::js_helpers::{set, vec_to_array};
use crate::point::Point;
use crate::render;
use crate::stroke;

pub(crate) fn generate_shape_mesh(
    kind: &str,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    size: f64,
) -> Vec<f64> {
    shape_mesh(kind, x1, y1, x2, y2, size, 32)
}

#[wasm_bindgen]
pub fn tools() -> Array {
    Array::of3(&"select".into(), &"draw".into(), &"eraser".into()).concat(&Array::of4(
        &"rect".into(),
        &"circle".into(),
        &"line".into(),
        &"arrow".into(),
    ))
}

#[wasm_bindgen]
pub fn process_stroke(
    data: Vec<f64>,
    epsilon: f64,
    segments: usize,
    base_size: f64,
    vel_influence: f64,
) -> Vec<f64> {
    let points: Vec<Point> = data
        .chunks_exact(3)
        .map(|c| Point::new(c[0], c[1], c[2]))
        .collect();
    if points.is_empty() {
        return Vec::new();
    }
    let mut s = stroke::Stroke::new_with_points(points);
    s.simplify_epsilon = epsilon;
    s.smooth_segments = segments;
    s.process_with_widths(base_size, vel_influence)
        .into_iter()
        .flatten()
        .collect()
}

#[wasm_bindgen]
pub fn mesh_from_centerline(data: Vec<f64>) -> Vec<f64> {
    let cl: Vec<[f64; 3]> = data.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
    geometry::generate_mesh(&cl, 8)
}

#[wasm_bindgen]
pub fn shape_mesh(
    kind: &str,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    width: f64,
    segments: usize,
) -> Vec<f64> {
    match kind {
        "line" => geometry::line_mesh(x1, y1, x2, y2, width),
        "rect" => geometry::rect_mesh(x1, y1, x2, y2, width),
        "circle" => {
            let cx = (x1 + x2) / 2.0;
            let cy = (y1 + y2) / 2.0;
            let rx = (x2 - x1).abs() / 2.0;
            let ry = (y2 - y1).abs() / 2.0;
            geometry::circle_mesh(
                cx,
                cy,
                rx.max(ry).max(1.0),
                rx.max(ry).max(1.0),
                width,
                segments,
            )
        }
        "arrow" => geometry::arrow_mesh(x1, y1, x2, y2, width),
        _ => Vec::new(),
    }
}

#[wasm_bindgen]
pub fn finalize_stroke(
    raw: Vec<f64>,
    color: &str,
    size: f64,
    id: &str,
    user_id: &str,
    epsilon: f64,
    segments: usize,
    vel_influence: f64,
) -> JsValue {
    if raw.len() < 3 {
        return JsValue::NULL;
    }
    let obj = js_sys::Object::new();
    set(&obj, "color", &color.into());
    set(&obj, "size", &Number::from(size));
    set(&obj, "id", &id.into());
    set(&obj, "userId", &user_id.into());

    if raw.len() == 3 {
        set(&obj, "type", &"dot".into());
        set(&obj, "x", &Number::from(raw[0]));
        set(&obj, "y", &Number::from(raw[1]));
        set(&obj, "pressure", &Number::from(raw[2]));
    } else {
        let points: Vec<Point> = raw
            .chunks_exact(3)
            .map(|c| Point::new(c[0], c[1], c[2]))
            .collect();
        let mut s = stroke::Stroke::new_with_points(points);
        s.simplify_epsilon = epsilon;
        s.smooth_segments = segments;
        let centerline = s.process_with_widths(size, vel_influence);
        let flat: Vec<f64> = centerline.iter().flat_map(|c| c.iter()).copied().collect();
        let mesh = geometry::generate_mesh(&centerline, 8);
        set(&obj, "type", &"path".into());
        set(&obj, "data", &vec_to_array(&flat));
        set(&obj, "mesh", &vec_to_array(&mesh));
    }
    JsValue::from(obj)
}

#[wasm_bindgen]
pub fn hit_path(px: f64, py: f64, data: Vec<f64>, width: f64) -> bool {
    geometry::hit_path(px, py, &data, width)
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn hit_shape(
    px: f64,
    py: f64,
    kind: &str,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    width: f64,
) -> bool {
    geometry::hit_shape(px, py, kind, x1, y1, x2, y2, width)
}

#[wasm_bindgen]
pub fn get_bounds(
    kind: &str,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    width: f64,
    data: Vec<f64>,
) -> Vec<f64> {
    geometry::get_bounds(kind, x1, y1, x2, y2, width, &data)
}

#[wasm_bindgen]
pub fn render_draw_mesh(ctx: &CanvasRenderingContext2d, data: Vec<f64>, color: &str) {
    render::draw_mesh(ctx, &data, color);
}

#[wasm_bindgen]
pub fn render_draw_dot(
    ctx: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    color: &str,
    size: f64,
    pressure: f64,
) {
    render::draw_dot(ctx, x, y, color, size, pressure);
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn render_draw_shape_canvas(
    ctx: &CanvasRenderingContext2d,
    kind: &str,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: &str,
    width: f64,
) {
    render::draw_shape_canvas(ctx, kind, x1, y1, x2, y2, color, width);
}

#[wasm_bindgen]
pub fn render_draw_cursor(ctx: &CanvasRenderingContext2d, x: f64, y: f64, color: &str) {
    render::draw_cursor_marker(ctx, x, y, color);
}

#[wasm_bindgen]
pub fn render_draw_selection(
    ctx: &CanvasRenderingContext2d,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    zoom: f64,
) {
    render::draw_selection_box(ctx, x1, y1, x2, y2, zoom);
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn render_draw_grid(
    ctx: &CanvasRenderingContext2d,
    show_grid: bool,
    bg_color: &str,
    cam_x: f64,
    cam_y: f64,
    cam_zoom: f64,
    canvas_w: f64,
    canvas_h: f64,
    grid: f64,
) {
    let p = render::GridParams {
        ctx,
        show_grid,
        bg_color,
        cam_x,
        cam_y,
        cam_zoom,
        canvas_w,
        canvas_h,
        grid,
    };
    render::draw_grid(&p);
}

#[wasm_bindgen]
pub fn tick_animation(remote_cursors: &JsValue, live_shapes: &JsValue) -> u8 {
    render::tick_animation(remote_cursors, live_shapes)
}
