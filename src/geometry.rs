use std::f64::consts::PI;

fn normal(ax: f64, ay: f64, bx: f64, by: f64) -> (f64, f64) {
    let dx = bx - ax;
    let dy = by - ay;
    let len = dx.hypot(dy);
    if len < 1e-10 {
        return (0.0, 0.0);
    }
    (-dy / len, dx / len)
}

fn angle_from_vector(x: f64, y: f64) -> f64 {
    y.atan2(x)
}

#[allow(clippy::too_many_arguments)]
fn cap_arc(
    cx: f64,
    cy: f64,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    cw: bool,
    segments: usize,
    out: &mut Vec<[f64; 2]>,
) {
    if segments == 0 {
        return;
    }
    for i in 0..=segments {
        let t = i as f64 / segments as f64;
        let angle = if cw {
            start_angle - t * (start_angle - end_angle)
        } else {
            start_angle + t * (end_angle - start_angle)
        };
        out.push([cx + radius * angle.cos(), cy + radius * angle.sin()]);
    }
}

fn add_tri(out: &mut Vec<[f64; 2]>, a: [f64; 2], b: [f64; 2], c: [f64; 2]) {
    out.push(a);
    out.push(b);
    out.push(c);
}

pub fn generate_mesh(points: &[[f64; 3]], cap_segments: usize) -> Vec<f64> {
    generate_mesh_inner(points, cap_segments, false)
}

pub fn generate_mesh_closed(points: &[[f64; 3]], cap_segments: usize) -> Vec<f64> {
    generate_mesh_inner(points, cap_segments, true)
}

fn generate_mesh_inner(points: &[[f64; 3]], cap_segments: usize, closed: bool) -> Vec<f64> {
    let n = points.len();
    if n < 2 || (closed && n < 3) {
        return Vec::new();
    }

    let cap = cap_segments.max(3);

    let mut left = Vec::with_capacity(n);
    let mut right = Vec::with_capacity(n);
    let mut dir_angles = Vec::with_capacity(n);

    let prev = |i: usize| {
        if i == 0 && closed {
            n - 2
        } else {
            i.saturating_sub(1)
        }
    };
    let next = |i: usize| {
        if i + 1 >= n {
            if closed { 1 } else { i }
        } else {
            i + 1
        }
    };

    for i in 0..n {
        let pi = prev(i);
        let ni = next(i);
        let (nx, ny) = if pi == i || ni == i {
            normal(points[i][0], points[i][1], points[ni][0], points[ni][1])
        } else {
            let (nx1, ny1) = normal(points[pi][0], points[pi][1], points[i][0], points[i][1]);
            let (nx2, ny2) = normal(points[i][0], points[i][1], points[ni][0], points[ni][1]);
            ((nx1 + nx2) * 0.5, (ny1 + ny2) * 0.5)
        };

        let hw = points[i][2] * 0.5;
        left.push([points[i][0] + nx * hw, points[i][1] + ny * hw]);
        right.push([points[i][0] - nx * hw, points[i][1] - ny * hw]);

        let angle = angle_from_vector(points[ni][0] - points[i][0], points[ni][1] - points[i][1]);
        dir_angles.push(angle);
    }

    let mut triangles: Vec<[f64; 2]> = Vec::new();

    if !closed {
        let r_angle = dir_angles[0] - PI / 2.0;
        let l_angle = dir_angles[0] + PI / 2.0;
        let hw0 = points[0][2] * 0.5;
        let mut arc_start = Vec::new();
        cap_arc(
            points[0][0],
            points[0][1],
            hw0,
            r_angle,
            l_angle,
            false,
            cap,
            &mut arc_start,
        );
        for i in 0..arc_start.len() - 1 {
            add_tri(
                &mut triangles,
                [points[0][0], points[0][1]],
                arc_start[i],
                arc_start[i + 1],
            );
        }
    }

    let seg_count = if closed { n } else { n - 1 };
    for i in 0..seg_count {
        let nxt = if i + 1 >= n { 0 } else { i + 1 };
        add_tri(&mut triangles, left[i], right[i], left[nxt]);
        add_tri(&mut triangles, right[i], right[nxt], left[nxt]);
    }

    if !closed {
        let last = n - 1;
        let r_angle_end = dir_angles[last] - PI / 2.0;
        let l_angle_end = dir_angles[last] + PI / 2.0;
        let hw_last = points[last][2] * 0.5;
        let mut arc_end = Vec::new();
        cap_arc(
            points[last][0],
            points[last][1],
            hw_last,
            l_angle_end,
            r_angle_end,
            true,
            cap,
            &mut arc_end,
        );
        for i in 0..arc_end.len() - 1 {
            add_tri(
                &mut triangles,
                [points[last][0], points[last][1]],
                arc_end[i],
                arc_end[i + 1],
            );
        }
    }

    triangles.into_iter().flatten().collect()
}

fn uniform_centerline(pts: &[[f64; 2]], width: f64) -> Vec<[f64; 3]> {
    pts.iter().map(|&p| [p[0], p[1], width]).collect()
}

pub fn line_mesh(x1: f64, y1: f64, x2: f64, y2: f64, width: f64) -> Vec<f64> {
    let pts = uniform_centerline(&[[x1, y1], [x2, y2]], width);
    generate_mesh_inner(&pts, 8, false)
}

pub fn rect_mesh(x1: f64, y1: f64, x2: f64, y2: f64, width: f64) -> Vec<f64> {
    let l = x1.min(x2);
    let r = x1.max(x2);
    let t = y1.min(y2);
    let b = y1.max(y2);
    let pts = uniform_centerline(&[[l, t], [r, t], [r, b], [l, b], [l, t]], width);
    generate_mesh_inner(&pts, 8, true)
}

pub fn circle_mesh(cx: f64, cy: f64, rx: f64, ry: f64, width: f64, segments: usize) -> Vec<f64> {
    let segs = segments.clamp(12, 128);
    let mut pts = Vec::with_capacity(segs + 1);
    for i in 0..segs {
        let a = i as f64 * 2.0 * PI / segs as f64;
        pts.push([cx + rx * a.cos(), cy + ry * a.sin(), width]);
    }
    pts.push([pts[0][0], pts[0][1], width]);
    generate_mesh_inner(&pts, 8, true)
}

pub fn arrow_mesh(x1: f64, y1: f64, x2: f64, y2: f64, width: f64) -> Vec<f64> {
    let mut base = line_mesh(x1, y1, x2, y2, width);
    let hl = (width * 3.0).max(8.0);
    let ha = PI / 6.0;
    let angle = (y2 - y1).atan2(x2 - x1);
    let tip = [x2, y2];
    let lw = [x2 - hl * (angle - ha).cos(), y2 - hl * (angle - ha).sin()];
    let rw = [x2 - hl * (angle + ha).cos(), y2 - hl * (angle + ha).sin()];
    let mut head = Vec::new();
    add_tri(&mut head, tip, lw, rw);
    base.extend(head.into_iter().flatten().collect::<Vec<f64>>());
    base
}

fn dist_to_segment_sq(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let ls = dx * dx + dy * dy;
    if ls == 0.0 {
        let dx = px - x1;
        let dy = py - y1;
        return dx * dx + dy * dy;
    }
    let t = ((px - x1) * dx + (py - y1) * dy) / ls;
    let t = t.clamp(0.0, 1.0);
    let px2 = x1 + t * dx;
    let py2 = y1 + t * dy;
    (px - px2) * (px - px2) + (py - py2) * (py - py2)
}

pub fn hit_path(px: f64, py: f64, data: &[f64], width: f64) -> bool {
    let t = (width * 0.5).max(4.0);
    let tsq = t * t;
    let mut i = 3;
    while i + 2 < data.len() {
        if dist_to_segment_sq(px, py, data[i - 3], data[i - 2], data[i], data[i + 1]) <= tsq {
            return true;
        }
        i += 3;
    }
    false
}

#[allow(clippy::too_many_arguments)]
pub fn hit_shape(
    px: f64,
    py: f64,
    kind: &str,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    width: f64,
) -> bool {
    let t = (width * 0.5).max(4.0);
    match kind {
        "rect" => {
            let l = x1.min(x2);
            let r = x1.max(x2);
            let u = y1.min(y2);
            let d = y1.max(y2);
            let cx = px.clamp(l, r);
            let cy = py.clamp(u, d);
            let dx = px - cx;
            let dy = py - cy;
            dx * dx + dy * dy <= t * t
        }
        "circle" => {
            let cx = (x1 + x2) / 2.0;
            let cy = (y1 + y2) / 2.0;
            let r = (x2 - x1).hypot(y2 - y1) / 2.0;
            let dist = (px - cx).hypot(py - cy);
            dist <= r || (dist - r).abs() <= t
        }
        "line" | "arrow" => dist_to_segment_sq(px, py, x1, y1, x2, y2) <= t * t,
        "dot" => (px - x1).hypot(py - y1) <= t,
        _ => false,
    }
}

pub fn get_bounds(
    kind: &str,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    width: f64,
    data: &[f64],
) -> Vec<f64> {
    match kind {
        "dot" => {
            vec![x1 - width, y1 - width, x1 + width, y1 + width]
        }
        "rect" | "line" | "arrow" => {
            vec![x1.min(x2), y1.min(y2), x1.max(x2), y1.max(y2)]
        }
        "circle" => {
            let cx = (x1 + x2) / 2.0;
            let cy = (y1 + y2) / 2.0;
            let r = (x2 - x1).hypot(y2 - y1) / 2.0;
            vec![cx - r, cy - r, cx + r, cy + r]
        }
        "path" => {
            if data.len() < 3 {
                return vec![0.0, 0.0, 0.0, 0.0];
            }
            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;
            for chunk in data.chunks_exact(3) {
                let x = chunk[0];
                let y = chunk[1];
                if x < min_x {
                    min_x = x;
                }
                if x > max_x {
                    max_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }
            let pad = width;
            vec![min_x - pad, min_y - pad, max_x + pad, max_y + pad]
        }
        _ => vec![0.0, 0.0, 0.0, 0.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_mesh_non_empty() {
        let m = line_mesh(0.0, 0.0, 10.0, 10.0, 2.0);
        assert!(!m.is_empty());
        assert!(m.len() % 6 == 0);
    }

    #[test]
    fn rect_mesh_non_empty() {
        let m = rect_mesh(0.0, 0.0, 10.0, 10.0, 2.0);
        assert!(!m.is_empty());
        assert!(m.len() % 6 == 0);
    }

    #[test]
    fn circle_mesh_non_empty() {
        let m = circle_mesh(0.0, 0.0, 5.0, 5.0, 2.0, 16);
        assert!(!m.is_empty());
        assert!(m.len() % 6 == 0);
    }

    #[test]
    fn arrow_mesh_has_head() {
        let m = arrow_mesh(0.0, 0.0, 10.0, 0.0, 2.0);
        assert!(!m.is_empty());
        assert!(m.len() % 6 == 0);
    }

    #[test]
    fn hit_path_on_centerline() {
        let data = vec![0.0, 0.0, 2.0, 10.0, 10.0, 2.0];
        assert!(hit_path(0.0, 0.0, &data, 2.0));
        assert!(hit_path(5.0, 5.0, &data, 2.0));
        assert!(!hit_path(0.0, 10.0, &data, 2.0));
    }

    #[test]
    fn generate_mesh_backward_compat() {
        let pts = vec![[0.0, 0.0, 2.0], [10.0, 0.0, 2.0]];
        let m = generate_mesh(&pts, 8);
        assert!(!m.is_empty());
    }

    #[test]
    fn closed_mesh_no_caps() {
        let pts = vec![
            [0.0, 0.0, 2.0],
            [10.0, 0.0, 2.0],
            [10.0, 10.0, 2.0],
            [0.0, 0.0, 2.0],
        ];
        let open = generate_mesh(&pts, 8).len();
        let closed = generate_mesh_closed(&pts, 8).len();
        assert!(closed < open);
    }

    #[test]
    fn hit_shape_rect_inside() {
        assert!(hit_shape(5.0, 5.0, "rect", 0.0, 0.0, 10.0, 10.0, 4.0));
    }

    #[test]
    fn hit_shape_rect_outside() {
        assert!(!hit_shape(20.0, 20.0, "rect", 0.0, 0.0, 10.0, 10.0, 4.0));
    }

    #[test]
    fn hit_shape_circle_center_is_hit() {
        assert!(hit_shape(5.0, 5.0, "circle", 0.0, 0.0, 10.0, 10.0, 2.0));
    }

    #[test]
    fn hit_shape_circle_on_perimeter() {
        let cx = (0.0 + 10.0) / 2.0;
        let cy = (0.0 + 10.0) / 2.0;
        let r = f64::hypot(10.0 - 0.0, 10.0 - 0.0) / 2.0;
        assert!(hit_shape(cx + r, cy, "circle", 0.0, 0.0, 10.0, 10.0, 4.0));
    }

    #[test]
    fn hit_shape_circle_outside() {
        assert!(!hit_shape(20.0, 20.0, "circle", 0.0, 0.0, 10.0, 10.0, 4.0));
    }

    #[test]
    fn hit_shape_line_on() {
        assert!(hit_shape(5.0, 5.0, "line", 0.0, 0.0, 10.0, 10.0, 8.0));
    }

    #[test]
    fn hit_shape_line_off() {
        assert!(!hit_shape(5.0, 5.0, "line", 0.0, 0.0, 10.0, 0.0, 2.0));
    }

    #[test]
    fn hit_shape_arrow_on() {
        assert!(hit_shape(5.0, 5.0, "arrow", 0.0, 0.0, 10.0, 10.0, 8.0));
    }

    #[test]
    fn hit_shape_dot_on() {
        assert!(hit_shape(0.0, 0.0, "dot", 0.0, 0.0, 0.0, 0.0, 10.0));
    }

    #[test]
    fn hit_shape_dot_off() {
        assert!(!hit_shape(100.0, 100.0, "dot", 0.0, 0.0, 0.0, 0.0, 10.0));
    }

    #[test]
    fn hit_shape_unknown_returns_false() {
        assert!(!hit_shape(0.0, 0.0, "triangle", 0.0, 0.0, 10.0, 10.0, 4.0));
    }

    #[test]
    fn get_bounds_dot() {
        let b = get_bounds("dot", 5.0, 5.0, 0.0, 0.0, 3.0, &[]);
        assert_eq!(b, vec![2.0, 2.0, 8.0, 8.0]);
    }

    #[test]
    fn get_bounds_rect() {
        let b = get_bounds("rect", 0.0, 0.0, 10.0, 20.0, 2.0, &[]);
        assert_eq!(b, vec![0.0, 0.0, 10.0, 20.0]);
    }

    #[test]
    fn get_bounds_rect_reversed() {
        let b = get_bounds("rect", 10.0, 20.0, 0.0, 0.0, 2.0, &[]);
        assert_eq!(b, vec![0.0, 0.0, 10.0, 20.0]);
    }

    #[test]
    fn get_bounds_circle() {
        let b = get_bounds("circle", 0.0, 0.0, 10.0, 10.0, 2.0, &[]);
        assert!(b[0] < 0.0);
        assert!(b[2] > 10.0);
    }

    #[test]
    fn get_bounds_line() {
        let b = get_bounds("line", 0.0, 0.0, 10.0, 5.0, 2.0, &[]);
        assert_eq!(b, vec![0.0, 0.0, 10.0, 5.0]);
    }

    #[test]
    fn get_bounds_arrow() {
        let b = get_bounds("arrow", 1.0, 2.0, 7.0, 9.0, 2.0, &[]);
        assert_eq!(b, vec![1.0, 2.0, 7.0, 9.0]);
    }

    #[test]
    fn get_bounds_path() {
        let data = vec![0.0, 0.0, 2.0, 5.0, 10.0, 2.0, 10.0, 5.0, 2.0];
        let b = get_bounds("path", 0.0, 0.0, 0.0, 0.0, 1.5, &data);
        assert!((b[0] - (-1.5)).abs() < 1e-10);
        assert!((b[1] - (-1.5)).abs() < 1e-10);
        assert!((b[2] - 11.5).abs() < 1e-10);
        assert!((b[3] - 11.5).abs() < 1e-10);
    }

    #[test]
    fn get_bounds_path_empty_data() {
        let b = get_bounds("path", 0.0, 0.0, 0.0, 0.0, 2.0, &[]);
        assert_eq!(b, vec![0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn get_bounds_unknown_returns_zeros() {
        let b = get_bounds("star", 1.0, 2.0, 3.0, 4.0, 5.0, &[]);
        assert_eq!(b, vec![0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn line_mesh_zero_width() {
        let m = line_mesh(0.0, 0.0, 10.0, 0.0, 0.0);
        assert!(m.is_empty() || m.len() >= 6);
    }

    #[test]
    fn rect_mesh_zero_area() {
        let m = rect_mesh(5.0, 5.0, 5.0, 5.0, 2.0);
        assert!(!m.is_empty());
        assert!(m.len() % 6 == 0);
    }

    #[test]
    fn circle_mesh_min_segments() {
        let m = circle_mesh(0.0, 0.0, 5.0, 5.0, 2.0, 3);
        assert!(!m.is_empty());
    }

    #[test]
    fn arrow_mesh_zero_length() {
        let m = arrow_mesh(5.0, 5.0, 5.0, 5.0, 2.0);
        assert!(!m.is_empty());
    }

    #[test]
    fn hit_path_with_single_segment() {
        let data = vec![0.0, 0.0, 4.0, 10.0, 10.0, 4.0];
        assert!(hit_path(5.0, 5.0, &data, 8.0));
        assert!(!hit_path(5.0, 20.0, &data, 8.0));
    }

    #[test]
    fn generate_mesh_no_points() {
        assert!(generate_mesh(&[], 8).is_empty());
    }

    #[test]
    fn generate_mesh_single_point() {
        assert!(generate_mesh(&[[5.0, 5.0, 2.0]], 8).is_empty());
    }

    #[test]
    fn generate_mesh_closed_too_few() {
        assert!(generate_mesh_closed(&[[0.0, 0.0, 2.0], [1.0, 1.0, 2.0]], 8).is_empty());
    }

    #[test]
    fn line_mesh_inverted_coords() {
        let m = line_mesh(10.0, 10.0, 0.0, 0.0, 2.0);
        assert!(!m.is_empty());
    }
}
