use wasm_bindgen::prelude::*;

use crate::point::Point;
use crate::simplify;
use crate::smooth;

#[wasm_bindgen]
pub fn process_stroke(data: Vec<f64>, epsilon: f64, segments: usize) -> Vec<f64> {
    let points: Vec<Point> = data
        .chunks_exact(3)
        .map(|c| Point::new(c[0], c[1], c[2]))
        .collect();

    if points.len() < 2 {
        return points.iter().flat_map(|p| vec![p.x, p.y]).collect();
    }

    let simplified = simplify::rdp(&points, epsilon);
    if simplified.len() < 2 {
        return simplified.iter().flat_map(|p| vec![p.x, p.y]).collect();
    }

    let smoothed = smooth::catmull_rom(&simplified, segments);
    smoothed.iter().flat_map(|p| vec![p.x, p.y]).collect()
}
