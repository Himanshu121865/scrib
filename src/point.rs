#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub pressure: f64,
}

impl Point {
    pub fn new(x: f64, y: f64, pressure: f64) -> Self {
        Point { x, y, pressure }
    }

    pub fn distance_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_point() {
        let p = Point::new(1.5, -2.5, 0.75);
        assert_eq!(p.x, 1.5);
        assert_eq!(p.y, -2.5);
        assert_eq!(p.pressure, 0.75);
    }

    #[test]
    fn zero_pressure_is_valid() {
        let p = Point::new(0.0, 0.0, 0.0);
        assert_eq!(p.pressure, 0.0);
    }

    #[test]
    fn full_pressure_is_valid() {
        let p = Point::new(0.0, 0.0, 1.0);
        assert_eq!(p.pressure, 1.0);
    }

    #[test]
    fn negative_coordinates() {
        let p = Point::new(-100.0, -200.0, 0.5);
        assert_eq!(p.x, -100.0);
        assert_eq!(p.y, -200.0);
    }

    #[test]
    fn distance_to_self_is_zero() {
        let p = Point::new(3.0, 4.0, 0.5);
        assert_eq!(p.distance_to(&p), 0.0);
    }

    #[test]
    fn distance_to_other_point() {
        let a = Point::new(0.0, 0.0, 0.5);
        let b = Point::new(3.0, 4.0, 0.5);
        assert!((a.distance_to(&b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn distance_is_symmetric() {
        let a = Point::new(1.0, 2.0, 0.3);
        let b = Point::new(4.0, 6.0, 0.7);
        assert!((a.distance_to(&b) - b.distance_to(&a)).abs() < 1e-10);
    }

    #[test]
    fn clone_eq() {
        let a = Point::new(1.0, 2.0, 0.5);
        let b = a;
        assert_eq!(a, b);
    }
}
