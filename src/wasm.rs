use wasm_bindgen::prelude::*;

use crate::point::Point;
use crate::stroke;
use crate::geometry;

#[wasm_bindgen]
pub fn process_stroke(data: Vec<f64>, epsilon: f64, segments: usize, base_size: f64) -> Vec<f64> {
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
    let result = s.process_with_widths(base_size);
    result.into_iter().flatten().collect()
}

#[wasm_bindgen]
pub fn mesh_from_centerline(data: Vec<f64>) -> Vec<f64> {
    let cl: Vec<[f64; 3]> = data
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
    geometry::generate_mesh(&cl, 8)
}

#[wasm_bindgen]
pub fn shape_mesh(
    kind: &str,
    x1: f64, y1: f64, x2: f64, y2: f64,
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
            let r = rx.max(ry).max(1.0);
            geometry::circle_mesh(cx, cy, r, r, width, segments)
        }
        "arrow" => geometry::arrow_mesh(x1, y1, x2, y2, width),
        _ => Vec::new(),
    }
}

#[wasm_bindgen]
pub fn hit_path(px: f64, py: f64, data: Vec<f64>, width: f64) -> bool {
    geometry::hit_path(px, py, &data, width)
}
