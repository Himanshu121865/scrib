use wasm_bindgen::prelude::*;

use crate::point::Point;
use crate::simplify;
use crate::smooth;

#[wasm_bindgen]
pub fn process_stroke(data: Vec<f64>, epsilon: f64, segments: usize, base_size: f64) -> Vec<f64> {
    let points: Vec<Point> = data
        .chunks_exact(3)
        .map(|c| Point::new(c[0], c[1], c[2]))
        .collect();

    if points.is_empty() {
        return Vec::new();
    }

    if points.len() < 2 {
        return points.iter().flat_map(|p| vec![p.x, p.y, base_size]).collect();
    }

    let simplified = simplify::rdp(&points, epsilon);
    if simplified.len() < 2 {
        return simplified.iter().flat_map(|p| vec![p.x, p.y, base_size]).collect();
    }

    let smoothed = smooth::catmull_rom(&simplified, segments);
    let widths = compute_widths(&smoothed, base_size);

    let mut result = Vec::with_capacity(smoothed.len() * 3);
    for (i, p) in smoothed.iter().enumerate() {
        result.push(p.x);
        result.push(p.y);
        result.push(widths[i]);
    }
    result
}

fn compute_widths(points: &[Point], base_size: f64) -> Vec<f64> {
    let n = points.len();
    if n < 2 {
        return vec![base_size; n];
    }

    // velocity = distance between consecutive points
    let mut vel = Vec::with_capacity(n);
    vel.push(0.0);
    for i in 1..n {
        let dx = points[i].x - points[i - 1].x;
        let dy = points[i].y - points[i - 1].y;
        vel.push((dx * dx + dy * dy).sqrt());
    }

    // smooth velocity with a 3-point moving average
    let mut smooth_vel = vel.clone();
    if n > 3 {
        for i in 1..n - 1 {
            smooth_vel[i] = (vel[i - 1] + vel[i] + vel[i + 1]) / 3.0;
        }
    }

    let max_v = smooth_vel.iter().cloned().fold(0.0, f64::max);

    // compute raw width: slow = thick, fast = thin
    let mut widths = Vec::with_capacity(n);
    for i in 0..n {
        let speed = if max_v > 0.0 {
            smooth_vel[i] / max_v
        } else {
            0.0
        };
        let p = points[i].pressure;
        // exaclidraw-like width curve
        let width = base_size * (0.15 + 0.85 * p * (1.0 - 0.75 * speed));
        widths.push(width);
    }

    // taper the start and end
    let taper = (n / 12).max(3).min(8);
    for i in 0..taper.min(n) {
        let t = i as f64 / taper as f64;
        widths[i] *= 0.1 + 0.9 * t;
    }
    for i in 0..taper.min(n) {
        let idx = n - 1 - i;
        let t = i as f64 / taper as f64;
        widths[idx] *= 0.1 + 0.9 * t;
    }

    widths
}
