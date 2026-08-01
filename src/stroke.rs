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

    pub fn process_with_widths(&self, base_size: f64, vel_influence: f64) -> Vec<[f64; 3]> {
        let pts = self.process();
        let widths = compute_widths(&pts, base_size, vel_influence);
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

pub fn compute_widths(points: &[Point], base_size: f64, vel_influence: f64) -> Vec<f64> {
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
        let width = base_size * (0.15 + 0.85 * p * (1.0 - vel_influence * speed));
        widths.push(width);
    }

    let taper = ((n / 12).clamp(3, 8)).min(n / 2);
    for (i, w) in widths.iter_mut().enumerate().take(taper) {
        let t = i as f64 / taper as f64;
        *w *= 0.1 + 0.9 * t;
    }
    for (i, w) in widths.iter_mut().rev().enumerate().take(taper) {
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
        let widths = compute_widths(&pts, 2.0, 0.75);
        assert_eq!(widths.len(), 3);
        assert!(widths.iter().all(|w| *w > 0.0));
    }

    #[test]
    fn process_with_widths_output() {
        let mut s = Stroke::new();
        s.add_point(Point::new(0.0, 0.0, 0.5));
        s.add_point(Point::new(5.0, 5.0, 0.5));
        s.add_point(Point::new(10.0, 0.0, 0.5));
        let out = s.process_with_widths(2.0, 0.75);
        assert!(out.len() > 3);
        assert_eq!(out[0].len(), 3);
    }

    #[test]
    fn zero_base_size_still_positive() {
        let pts = vec![Point::new(0.0, 0.0, 1.0), Point::new(1.0, 0.0, 1.0)];
        let widths = compute_widths(&pts, 0.0, 0.75);
        assert_eq!(widths.len(), 2);
        assert!(widths.iter().all(|w| *w >= 0.0));
    }

    #[test]
    fn zero_pressure_produces_thin_stroke() {
        let pts = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(10.0, 0.0, 0.0),
            Point::new(20.0, 0.0, 0.0),
        ];
        let widths = compute_widths(&pts, 10.0, 0.75);
        for w in &widths {
            assert!(*w < 10.0);
            assert!(*w >= 0.0);
        }
    }

    #[test]
    fn full_pressure_produces_thick_stroke() {
        let pts: Vec<Point> = (0..20).map(|_| Point::new(0.0, 0.0, 1.0)).collect();
        let widths = compute_widths(&pts, 10.0, 0.75);
        let max_w = widths.iter().copied().fold(0.0, f64::max);
        assert!(max_w > 8.0);
    }

    #[test]
    fn width_taper_at_ends() {
        let pts: Vec<Point> = (0..20).map(|i| Point::new(i as f64, 0.0, 1.0)).collect();
        let widths = compute_widths(&pts, 10.0, 0.75);
        assert!(widths[0] < widths[5]);
        assert!(widths[widths.len() - 1] < widths[widths.len() - 6]);
    }

    #[test]
    fn high_velocity_reduces_width() {
        let pts = vec![
            Point::new(0.0, 0.0, 1.0),
            Point::new(100.0, 0.0, 1.0),
            Point::new(200.0, 0.0, 1.0),
        ];
        let widths = compute_widths(&pts, 10.0, 0.75);
        let min_w = widths.iter().copied().fold(f64::INFINITY, f64::min);
        assert!(min_w > 0.0);
        assert!(min_w < 10.0);
    }

    #[test]
    fn many_points_does_not_panic() {
        let mut s = Stroke::new();
        s.simplify_epsilon = 0.02;
        s.smooth_segments = 4;
        for i in 0..1000 {
            let x = i as f64 * 0.1;
            s.add_point(Point::new(x, (x * 0.5).sin(), 0.5));
        }
        let out = s.process_with_widths(4.0, 0.75);
        assert!(out.len() > 200);
    }

    #[test]
    fn default_stroke_has_default_values() {
        let s = Stroke::new();
        assert!(s.points.is_empty());
        assert!((s.simplify_epsilon - 0.5).abs() < 1e-10);
        assert_eq!(s.smooth_segments, 4);
    }

    #[test]
    fn short_stroke_taper_no_double_apply() {
        let pts: Vec<Point> = (0..4).map(|_| Point::new(0.0, 0.0, 1.0)).collect();
        let widths = compute_widths(&pts, 10.0, 0.75);
        assert_eq!(widths.len(), 4);
        assert!(widths[1] > widths[0], "mid should be thicker than tip");
        assert!(widths[2] > widths[3], "mid should be thicker than tip");
        assert!((widths[1] / widths[2] - 1.0).abs() < 0.01, "symmetric");
    }

    #[test]
    fn taper_symmetric_both_ends() {
        let pts: Vec<Point> = (0..10).map(|_| Point::new(0.0, 0.0, 1.0)).collect();
        let widths = compute_widths(&pts, 10.0, 0.75);
        assert!((widths[0] / widths[widths.len() - 1] - 1.0).abs() < 0.01);
        assert!((widths[1] / widths[widths.len() - 2] - 1.0).abs() < 0.01);
        assert!((widths[2] / widths[widths.len() - 3] - 1.0).abs() < 0.01);
        assert!((widths[3] - widths[4]).abs() < 0.01);
    }

    #[test]
    fn pressure_zero_is_thinner_than_full() {
        let pts0: Vec<Point> = (0..5).map(|i| Point::new(i as f64, 0.0, 0.0)).collect();
        let pts1: Vec<Point> = (0..5).map(|i| Point::new(i as f64, 0.0, 1.0)).collect();
        let w0 = compute_widths(&pts0, 10.0, 0.75);
        let w1 = compute_widths(&pts1, 10.0, 0.75);
        for i in 0..5 {
            assert!(
                w0[i] < w1[i],
                "zero pressure should be thinner at index {i}"
            );
        }
    }

    #[test]
    fn default_trait_creates_empty() {
        let s: Stroke = Default::default();
        assert!(s.points.is_empty());
    }

    #[test]
    fn add_point_extends_points() {
        let mut s = Stroke::new();
        s.add_point(Point::new(1.0, 2.0, 0.5));
        s.add_point(Point::new(3.0, 4.0, 0.5));
        assert_eq!(s.points.len(), 2);
    }
}
