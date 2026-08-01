use crate::point::Point;

pub fn rdp(points: &[Point], epsilon: f64) -> Vec<Point> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut stack: Vec<(usize, usize)> = Vec::new();
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    stack.push((0, points.len() - 1));

    while let Some((start, end)) = stack.pop() {
        let first = &points[start];
        let last = &points[end];

        let mut max_dist = 0.0;
        let mut max_idx = start;

        for (offset, p) in points[start + 1..end].iter().enumerate() {
            let dist = perpendicular_distance(p, first, last);
            if dist > max_dist {
                max_dist = dist;
                max_idx = start + 1 + offset;
            }
        }

        if max_dist > epsilon {
            keep[max_idx] = true;
            stack.push((start, max_idx));
            stack.push((max_idx, end));
        }
    }

    let mut result = Vec::with_capacity(points.len());
    for (i, p) in points.iter().enumerate() {
        if keep[i] {
            result.push(*p);
        }
    }
    result
}

fn perpendicular_distance(point: &Point, line_start: &Point, line_end: &Point) -> f64 {
    let dx = line_end.x - line_start.x;
    let dy = line_end.y - line_start.y;
    let length_sq = dx * dx + dy * dy;

    if length_sq == 0.0 {
        return point.distance_to(line_start);
    }

    let t = ((point.x - line_start.x) * dx + (point.y - line_start.y) * dy) / length_sq;
    let t = t.clamp(0.0, 1.0);

    let proj_x = line_start.x + t * dx;
    let proj_y = line_start.y + t * dy;

    let dx = point.x - proj_x;
    let dy = point.y - proj_y;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_points() {
        assert_eq!(rdp(&[], 0.5), vec![]);
    }

    #[test]
    fn single_point() {
        let p = Point::new(0.0, 0.0, 0.5);
        assert_eq!(rdp(&[p], 0.5), vec![p]);
    }

    #[test]
    fn two_points() {
        let a = Point::new(0.0, 0.0, 0.5);
        let b = Point::new(1.0, 1.0, 0.5);
        assert_eq!(rdp(&[a, b], 0.5), vec![a, b]);
    }

    #[test]
    fn colinear_points() {
        let points = vec![
            Point::new(0.0, 0.0, 0.5),
            Point::new(0.5, 0.5, 0.5),
            Point::new(1.0, 1.0, 0.5),
        ];
        let result = rdp(&points, 0.1);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn retains_outlier() {
        let points = vec![
            Point::new(0.0, 0.0, 0.5),
            Point::new(5.0, 5.0, 0.5),
            Point::new(10.0, 0.0, 0.5),
        ];
        let result = rdp(&points, 2.0);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn horizontal_line_all_colinear() {
        let points: Vec<Point> = (0..100).map(|i| Point::new(i as f64, 0.0, 0.5)).collect();
        let result = rdp(&points, 0.1);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].x, 0.0);
        assert_eq!(result[1].x, 99.0);
    }

    #[test]
    fn vertical_line_all_colinear() {
        let points: Vec<Point> = (0..50).map(|i| Point::new(0.0, i as f64, 0.5)).collect();
        let result = rdp(&points, 0.1);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].y, 0.0);
        assert_eq!(result[1].y, 49.0);
    }

    #[test]
    fn zigzag_preserves_all() {
        let mut points = Vec::new();
        for i in 0..20 {
            points.push(Point::new(i as f64, (i % 2) as f64 * 10.0, 0.5));
        }
        let result = rdp(&points, 0.5);
        assert!(result.len() > 10);
    }

    #[test]
    fn epsilon_zero_preserves_all() {
        let points = vec![
            Point::new(0.0, 0.0, 0.5),
            Point::new(1.0, 0.1, 0.5),
            Point::new(2.0, 0.0, 0.5),
            Point::new(3.0, 0.1, 0.5),
        ];
        let result = rdp(&points, 0.0);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn large_epsilon_reduces_to_two() {
        let points = vec![
            Point::new(0.0, 0.0, 0.5),
            Point::new(3.0, 100.0, 0.5),
            Point::new(7.0, -50.0, 0.5),
            Point::new(10.0, 0.0, 0.5),
        ];
        let result = rdp(&points, 1000.0);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn epsilon_zero_many_points() {
        let points: Vec<Point> = (0..2000)
            .map(|i| Point::new(i as f64, (i % 3) as f64, 0.5))
            .collect();
        let result = rdp(&points, 0.0);
        assert!(!result.is_empty());
        assert_eq!(result[0].x, 0.0);
        assert_eq!(result.last().unwrap().x, 1999.0);
    }

    #[test]
    fn epsilon_small_colinear_preserves_first_and_last() {
        let points: Vec<Point> = (0..100).map(|i| Point::new(i as f64, 0.0, 0.5)).collect();
        let result = rdp(&points, 1e-12);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].x, 0.0);
        assert_eq!(result[1].x, 99.0);
    }

    #[test]
    fn duplicate_points() {
        let p = Point::new(5.0, 5.0, 0.5);
        let points = vec![p, p, p, p, p];
        let result = rdp(&points, 0.1);
        assert_eq!(result.len(), 2);
    }
}
