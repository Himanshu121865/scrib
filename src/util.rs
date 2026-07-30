use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn epsilon() -> f64 {
    0.5
}

#[wasm_bindgen]
pub fn segments() -> usize {
    6
}

#[wasm_bindgen]
pub fn grid_size() -> f64 {
    30.0
}

#[wasm_bindgen]
pub fn throttle_draw() -> f64 {
    80.0
}

#[wasm_bindgen]
pub fn throttle_cursor() -> f64 {
    40.0
}

#[wasm_bindgen]
pub fn cap_floats() -> usize {
    48
}

#[wasm_bindgen]
pub fn incr_throttle() -> f64 {
    30.0
}

#[wasm_bindgen]
pub fn velocity_influence() -> f64 {
    0.75
}

#[wasm_bindgen]
pub fn screen_to_canvas(sx: f64, sy: f64, cam_x: f64, cam_y: f64, cam_zoom: f64) -> Vec<f64> {
    vec![(sx - cam_x) / cam_zoom, (sy - cam_y) / cam_zoom]
}

#[wasm_bindgen]
pub fn dist_to_segment(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let ls = dx * dx + dy * dy;
    if ls == 0.0 {
        return (px - x1).hypot(py - y1);
    }
    let mut t = ((px - x1) * dx + (py - y1) * dy) / ls;
    t = t.clamp(0.0, 1.0);
    (px - (x1 + t * dx)).hypot(py - (y1 + t * dy))
}

#[wasm_bindgen]
pub fn snap(val: f64, show_grid: bool) -> f64 {
    if show_grid {
        (val / grid_size()).round() * grid_size()
    } else {
        val
    }
}
