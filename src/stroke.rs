use crate::point::Point;
use crate::simplify;
use crate::smooth;

pub struct Stroke {
    pub points: Vec<Point>,
    pub simplify_epsilon: f64,
    pub smooth_segments: usize,
}

impl Stroke {
    pub fn new() -> Self {
        Stroke {
            points: Vec::new(),
            simplify_epsilon: 0.5,
            smooth_segments: 4,
        }
    }

    pub fn new_with_points(points: Vec<Point>) -> Self {
        Stroke {
            points,
            simplify_epsilon: 0.5,
            smooth_segments: 4,
        }
    }

    pub fn add_point(&mut self, point: Point) {
        self.points.push(point);
    }

    pub fn process(&self) -> Vec<Point> {
        pipeline(&self.points, self.simplify_epsilon, self.smooth_segments)
    }

    pub fn process_with_widths(&self, base_size: f64) -> Vec<[f64; 3]> {
        let pts = self.process();
        let widths = compute_widths(&pts, base_size);
        pts.iter()
            .zip(widths.iter())
            .map(|(p, w)| [p.x, p.y, *w])
            .collect()
    }
}

impl Default for Stroke {
    fn default() -> Self {
        Self::new()
    }
}

pub fn pipeline(points: &[Point], epsilon: f64, segments: usize) -> Vec<Point> {
    if points.len() < 2 {
        return points.to_vec();
    }
    let simplified = simplify::rdp(points, epsilon);
    if simplified.len() < 2 {
        return simplified;
    }
    smooth::catmull_rom(&simplified, segments)
}

pub fn compute_widths(points: &[Point], base_size: f64) -> Vec<f64> {
    let n = points.len();
    if n < 2 {
        return vec![base_size; n];
    }

    let mut vel = Vec::with_capacity(n);
    vel.push(0.0);
    for i in 1..n {
        let dx = points[i].x - points[i - 1].x;
        let dy = points[i].y - points[i - 1].y;
        vel.push((dx * dx + dy * dy).sqrt());
    }

    let mut smooth_vel = vel.clone();
    if n > 3 {
        for i in 1..n - 1 {
            smooth_vel[i] = (vel[i - 1] + vel[i] + vel[i + 1]) / 3.0;
        }
    }

    let max_v = smooth_vel.iter().copied().fold(0.0, f64::max);

    let mut widths = Vec::with_capacity(n);
    for i in 0..n {
        let speed = if max_v > 0.0 {
            smooth_vel[i] / max_v
        } else {
            0.0
        };
        let p = points[i].pressure;
        let width = base_size * (0.15 + 0.85 * p * (1.0 - 0.75 * speed));
        widths.push(width);
    }

    let taper = (n / 12).clamp(3, 8);
    for (i, w) in widths.iter_mut().enumerate().take(taper.min(n)) {
        let t = i as f64 / taper as f64;
        *w *= 0.1 + 0.9 * t;
    }
    for (i, w) in widths.iter_mut().rev().enumerate().take(taper.min(n)) {
        let t = i as f64 / taper as f64;
        *w *= 0.1 + 0.9 * t;
    }

    widths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_stroke() {
        let stroke = Stroke::new();
        assert!(stroke.process().is_empty());
    }

    #[test]
    fn single_point_stroke() {
        let mut stroke = Stroke::new();
        stroke.add_point(Point::new(0.0, 0.0, 0.5));
        assert_eq!(stroke.process().len(), 1);
    }

    #[test]
    fn pipeline_produces_smooth_path() {
        let mut stroke = Stroke::new();
        stroke.simplify_epsilon = 0.3;
        stroke.smooth_segments = 4;
        for i in 0..10 {
            let x = i as f64;
            let y = (x * 0.5).sin();
            stroke.add_point(Point::new(x, y, 0.5));
        }
        let result = stroke.process();
        assert!(result.len() > 10);
    }

    #[test]
    fn widths_match_expected_count() {
        let pts = vec![
            Point::new(0.0, 0.0, 0.5),
            Point::new(1.0, 0.0, 0.5),
            Point::new(2.0, 0.0, 0.5),
        ];
        let widths = compute_widths(&pts, 2.0);
        assert_eq!(widths.len(), 3);
        assert!(widths.iter().all(|w| *w > 0.0));
    }

    #[test]
    fn process_with_widths_output() {
        let mut s = Stroke::new();
        s.add_point(Point::new(0.0, 0.0, 0.5));
        s.add_point(Point::new(5.0, 5.0, 0.5));
        s.add_point(Point::new(10.0, 0.0, 0.5));
        let out = s.process_with_widths(2.0);
        assert!(out.len() > 3);
        assert_eq!(out[0].len(), 3);
    }
}
