use crate::point::Point;

pub fn catmull_rom(points: &[Point], segments: usize) -> Vec<Point> {
    if points.len() < 2 {
        return points.to_vec();
    }
    if points.len() == 2 || segments == 0 {
        return points.to_vec();
    }

    let knots = compute_knots(points, 0.5);

    let mut result = Vec::with_capacity(points.len() * segments);

    for i in 0..points.len() - 1 {
        for s in 0..segments {
            let t = knots[i] + (knots[i + 1] - knots[i]) * (s as f64 / segments as f64);
            result.push(cr_eval(points, &knots, i, t));
        }
    }

    result.push(points[points.len() - 1]);
    result
}

fn compute_knots(points: &[Point], alpha: f64) -> Vec<f64> {
    let mut knots = Vec::with_capacity(points.len());
    knots.push(0.0);
    for i in 1..points.len() {
        let dx = points[i].x - points[i - 1].x;
        let dy = points[i].y - points[i - 1].y;
        let dist = (dx * dx + dy * dy).sqrt();
        knots.push(knots[i - 1] + dist.powf(alpha));
    }
    for i in 1..knots.len() {
        if knots[i] <= knots[i - 1] {
            knots[i] = knots[i - 1] + 1e-10;
        }
    }
    knots
}

fn cr_eval(points: &[Point], knots: &[f64], i: usize, t: f64) -> Point {
    let n = points.len();

    let p1 = points[i];
    let p2 = points[i + 1];
    let t1 = knots[i];
    let t2 = knots[i + 1];

    let (p0, t0) = if i > 0 {
        (points[i - 1], knots[i - 1])
    } else {
        (points[0], 2.0 * knots[0] - knots[1])
    };

    let (p3, t3) = if i + 2 < n {
        (points[i + 2], knots[i + 2])
    } else {
        (points[n - 1], 2.0 * knots[n - 1] - knots[n - 2])
    };

    let l0 = lagrange(t, t0, t1, t2, t3);
    let l1 = lagrange(t, t1, t0, t2, t3);
    let l2 = lagrange(t, t2, t0, t1, t3);
    let l3 = lagrange(t, t3, t0, t1, t2);

    let x = p0.x * l0 + p1.x * l1 + p2.x * l2 + p3.x * l3;
    let y = p0.y * l0 + p1.y * l1 + p2.y * l2 + p3.y * l3;
    let pressure = p1.pressure + (p2.pressure - p1.pressure) * ((t - t1) / (t2 - t1));

    Point { x, y, pressure }
}

fn lagrange(t: f64, t_i: f64, t_a: f64, t_b: f64, t_c: f64) -> f64 {
    ((t - t_a) * (t - t_b) * (t - t_c)) / ((t_i - t_a) * (t_i - t_b) * (t_i - t_c))
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
