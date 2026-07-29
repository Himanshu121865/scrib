use crate::point::Point;

pub fn rdp(points: &[Point], epsilon: f64) -> Vec<Point> {
    // Ramer-Douglas-Peucker: keep the farthest outlier, recurse on both sides
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut max_dist = 0.0;
    let mut max_idx = 0;

    let first = &points[0];
    let last = &points[points.len() - 1];

    for (i, point) in points
        .iter()
        .enumerate()
        .skip(1)
        .take(points.len().saturating_sub(2))
    {
        let dist = perpendicular_distance(point, first, last);
        if dist > max_dist {
            max_dist = dist;
            max_idx = i;
        }
    }

    if max_dist > epsilon {
        let mut left = rdp(&points[..=max_idx], epsilon);
        let right = rdp(&points[max_idx..], epsilon);
        left.pop();
        left.extend(right);
        left
    } else {
        vec![*first, *last]
    }
}

fn perpendicular_distance(point: &Point, line_start: &Point, line_end: &Point) -> f64 {
    // Distance from point to the segment line_start..line_end
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
}
