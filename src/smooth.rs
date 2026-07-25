use crate::point::Point;

pub fn catmull_rom(points: &[Point], segments: usize) -> Vec<Point> {
    if points.len() < 2 {
        return points.to_vec();
    }
    if points.len() == 2 || segments == 0 {
        return points.to_vec();
    }

    let mut result = Vec::with_capacity(points.len() * segments);

    for i in 0..points.len() - 1 {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 < points.len() {
            points[i + 2]
        } else {
            points[points.len() - 1]
        };

        for s in 0..segments {
            let t = s as f64 / segments as f64;
            result.push(interpolate(p0, p1, p2, p3, t));
        }
    }

    result.push(points[points.len() - 1]);
    result
}

fn interpolate(p0: Point, p1: Point, p2: Point, p3: Point, t: f64) -> Point {
    let t2 = t * t;
    let t3 = t2 * t;

    let x = 0.5 * ((2.0 * p1.x)
        + (-p0.x + p2.x) * t
        + (2.0 * p0.x - 5.0 * p1.x + 4.0 * p2.x - p3.x) * t2
        + (-p0.x + 3.0 * p1.x - 3.0 * p2.x + p3.x) * t3);

    let y = 0.5 * ((2.0 * p1.y)
        + (-p0.y + p2.y) * t
        + (2.0 * p0.y - 5.0 * p1.y + 4.0 * p2.y - p3.y) * t2
        + (-p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y) * t3);

    let pressure = p1.pressure + (p2.pressure - p1.pressure) * t;

    Point { x, y, pressure }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_points() {
        assert_eq!(catmull_rom(&[], 4), vec![]);
    }

    #[test]
    fn single_point() {
        let p = Point::new(1.0, 2.0, 0.5);
        assert_eq!(catmull_rom(&[p], 4), vec![p]);
    }

    #[test]
    fn two_points() {
        let a = Point::new(0.0, 0.0, 0.5);
        let b = Point::new(1.0, 1.0, 0.5);
        assert_eq!(catmull_rom(&[a, b], 4), vec![a, b]);
    }

    #[test]
    fn produces_more_points() {
        let points = vec![
            Point::new(0.0, 0.0, 0.5),
            Point::new(1.0, 1.0, 0.5),
            Point::new(2.0, 0.0, 0.5),
        ];
        let result = catmull_rom(&points, 4);
        assert!(result.len() > points.len());
    }

    #[test]
    fn preserves_endpoints() {
        let points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(5.0, 5.0, 0.5),
            Point::new(10.0, 0.0, 1.0),
        ];
        let result = catmull_rom(&points, 4);
        assert_eq!(result.first().unwrap(), &points[0]);
        assert_eq!(result.last().unwrap(), &points[points.len() - 1]);
    }
}
