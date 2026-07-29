use wasm_bindgen::prelude::*;

use crate::point::Point;
use crate::stroke;

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
