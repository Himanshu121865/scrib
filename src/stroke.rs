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

    pub fn add_point(&mut self, point: Point) {
        self.points.push(point);
    }

    pub fn process(&self) -> Vec<Point> {
        if self.points.len() < 2 {
            return self.points.clone();
        }
        let simplified = simplify::rdp(&self.points, self.simplify_epsilon);
        if simplified.len() < 2 {
            return simplified;
        }
        smooth::catmull_rom(&simplified, self.smooth_segments)
    }
}

impl Default for Stroke {
    fn default() -> Self {
        Self::new()
    }
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
}
