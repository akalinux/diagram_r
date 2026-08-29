use js_sys::Number;

use crate::{
    Point, Transform,
    constants::{AREA_SCALE_EPSILON, HALF, R_90, R_180, R_270, R_360},
    square::Corners,
};

pub type AngleNorthSouth = (f32, f32, f32);
pub type FullBox = (Point, Point, Point, Point);
pub fn to_map_xy(p: &Point, t: &Transform) -> Point {
    let px = p.x - t.x;
    let py = p.y - t.y;
    let x = px / t.k;
    let y = py / t.k;
    Point { x, y }
}

pub fn to_fixed_px(n: f32) -> String {
    let js_num: Number = n.into();
    let js_str = unsafe { js_num.to_fixed(2).unwrap_unchecked() };
    let mut str = String::from(js_str);
    str.push_str("px");
    str
}

pub fn get_xy_r(cx: f32, cy: f32, r: f32, rad: f32) -> Point {
    let x = cx + r * rad.cos();
    let y = cy + r * rad.sin();
    Point { x, y }
}

pub fn get_radians(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x1 - x2;
    let dy = y1 - y2;
    let base = dy.atan2(dx);
    match base < 0.0 {
        true => base + R_360,
        false => base,
    }
}

pub fn triangle_area(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> f32 {
    return (x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2)).abs() * HALF;
}

/// Provides the [Corners] from the given 4 [Point] instances.
pub fn compute_line_box(ne: &Point, points: &[&Point]) -> Corners {
    let mut min_x = ne.x;
    let mut max_x = ne.x;
    let mut min_y = ne.y;
    let mut max_y = ne.y;
    for p in points {
        if max_x < p.x {
            max_x = p.x;
        }
        if max_y < p.y {
            max_y = p.y;
        }
        if min_x > p.x {
            min_x = p.x;
        }
        if min_y > p.y {
            min_y = p.y;
        }
    }
    (min_x, max_x, min_y, max_y)
}

pub fn north_box_from(a: &Point, b: &Point, r: f32) -> (Point, Point, Point, f32, f32) {
    let rad = a.get_radians(b);
    let north = rad + R_90;
    let nw = get_xy_r(a.x, a.y, r, north);
    let distance = nw.get_move_distance(&a);

    let ne = b.sub_distance(&distance);
    (nw, ne, distance, north, rad)
}
pub fn full_box_from(a: &Point, b: &Point, r: f32) -> (FullBox, (Point, f32, f32)) {
    let (nw, ne, distance, north, angle) = north_box_from(a, b, r);

    let sw = a.add_distance(&distance);
    let se = b.add_distance(&distance);
    ((nw, ne, sw, se), (distance, north, angle))
}

pub fn inside_box(pbox: &FullBox, p: &Point) -> bool {
    let (nw, ne, sw, se) = pbox;
    let box_area = (triangle_area(ne.x, ne.y, nw.x, nw.y, se.x, se.y)
        + triangle_area(ne.x, ne.y, nw.x, nw.y, sw.x, sw.y))
        * AREA_SCALE_EPSILON;

    let mut triangle_sum = 0.0;
    let order = [&ne, &nw, &sw, &se, &ne];
    for id in 0..order.len() - 1 {
        let left = order[id];
        let right = order[id + 1];
        triangle_sum += triangle_area(p.x, p.y, left.x, left.y, right.x, right.y);
        if triangle_sum > box_area {
            return false;
        };
    }
    return true;
}

pub fn get_distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    return get_distance_square(x1, y1, x2, y2).sqrt();
}

pub fn get_distance_square(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    return (x1 - x2).powi(2) + (y1 - y2).powi(2);
}

pub fn compute_r_for_even_space_on_circle(r: f32, points: f32) -> f32 {
    let rad = R_360 / points;
    let a = get_xy_r(0.0, 0.0, r, 0.0);
    let b = get_xy_r(0.0, 0.0, r, rad);
    let cmp = get_distance(a.x, a.y, b.x, b.y);
    let scale = r / cmp;
    return r * scale;
}

pub fn inside_circle(p: &Point, c: &Point, r: f32) -> bool {
    return (p.x - c.x).powi(2) + (p.y - c.y).powi(2) <= r.powi(2);
}

/// Takes a point from the map and converts it to a point for the screen.
pub fn to_screen_xy(p: &Point, t: &Transform) -> Point {
    let px = p.x + t.x;
    let py = p.y + t.y;
    let x = px * t.k;
    let y = py * t.k;
    return Point { x, y };
}

pub fn rad_needs_normalization(angle: f32) -> bool {
    angle >= R_90 && angle <= R_270
}

pub fn normalize_rad(rad: f32) -> (f32, bool) {
    match rad_needs_normalization(rad) {
        true => (rad + R_180, true),
        false => (rad, false),
    }
}

pub fn get_abc_from_points(begin: &Point, end: &Point) -> (f32, f32, f32) {
    let a = end.y - begin.y;
    let b = begin.x - end.x;
    let c = a * begin.x + b * begin.y;
    (a, b, c)
}

pub fn get_intersection(
    start1: &Point,
    end1: &Point,
    start2: &Point,
    end2: &Point,
) -> Option<Point> {
    let (a1, b1, c1) = get_abc_from_points(start1, end1);
    let (a2, b2, c2) = get_abc_from_points(start2, end2);

    let d = a1 * b2 - a2 * b1;
    if d.abs() < f32::EPSILON {
        return None;
    }
    let x = (c1 * b2 - c2 * b1) / d;
    let y = (a1 * c2 - a2 * c1) / d;
    Some(Point { x, y })
}

pub fn force_intersection(start1: &Point, end1: &Point, start2: &Point, end2: &Point) -> Point {
    match get_intersection(start1, end1, start2, end2) {
        Some(p) => p,
        None => start1.get_center(start2),
    }
}

pub fn compute_arc_point(t: f32, s: &Point, c: &Point, e: &Point) -> Point {
    let a = s.add_distance(&s.get_move_distance(c).scale(t));
    let b = c.add_distance(&c.get_move_distance(e).scale(t));
    println!("{},{}", a, b);

    a.get_center(&b)
}

pub fn inside_arc(s: &Point, c: &Point, e: &Point, check: &Point) -> bool {
    // 1. Shift relative to start point (p0)
    let cx = c.x - s.x;
    let cy = c.y - s.y;
    let ex = e.x - s.x;
    let ey = e.y - s.y;
    let px = check.x - s.x;
    let py = check.y - s.y;

    // 2. Compute a, b, c, d
    let a = 2.0 * cx;
    let b = 2.0 * cy;
    let c = ex - (2.0 * cx);
    let d = ey - (2.0 * cy);

    // 3. Compute the determinant
    let det = a * d - b * c;

    // 4. Calculate dynamic tolerance
    let max_term = a.abs().max(b.abs()).max(c.abs()).max(d.abs());
    let scale = (max_term * max_term).max(1.0);
    let dynamic_tolerance = 1e-6 * scale;

    // 5. STRAIGHT LINE FALLBACK
    if det.abs() < dynamic_tolerance {
        // Compute the squared length of the baseline segment (p0 to p2)
        let segment_len_sq = ex * ex + ey * ey;

        // If start and end points are the exact same point
        if segment_len_sq < 1e-6 {
            let dist_sq = px * px + py * py;
            return dist_sq < 1e-6; // Inside if it matches the single point
        }

        // Project the test point onto the baseline to find the parameter 't'
        // t = (vector_p0_to_test DOT vector_p0_to_p2) / length_squared
        let t = (px * ex + py * ey) / segment_len_sq;

        // Ensure the projection falls within the segment bounds [0.0, 1.0]
        if t >= 0.0 && t <= 1.0 {
            // Find the closest point on the line segment
            let closest_x = t * ex;
            let closest_y = t * ey;

            // Calculate distance from test point to the closest point
            let dx = px - closest_x;
            let dy = py - closest_y;
            let distance_sq = dx * dx + dy * dy;

            // Scale line tolerance based on segment length
            let line_tolerance = 1e-6 * segment_len_sq.max(1.0);
            return distance_sq < line_tolerance;
        }
        return false;
    }

    // 6. Standard curve tracking if not a straight line
    let u = (d * px - c * py) / det;
    let v = (-b * px + a * py) / det;
    let f = u * u - v;

    f < 0.0 && u >= 0.0 && u <= 1.0
}

pub fn compute_arc_line_boundries(a: &Point, c: &Point, b: &Point, r: f32) -> [Point; 6] {
    let d = a.get_point(b, r, R_90).get_move_distance(a);

    [
        a.sub_distance(&d),
        c.sub_distance(&d),
        b.sub_distance(&d),
        b.add_distance(&d),
        c.sub_distance(&d),
        a.add_distance(&d),
    ]
}
