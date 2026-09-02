use js_sys::Number;

use crate::{
    LabelPosition, Point, Transform,
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
    // this is 16 steps can be reduced to 8 with simd
    let a = s.add_distance(&s.get_move_distance(c).scale(t));
    let b = c.add_distance(&c.get_move_distance(e).scale(t));

    a.add_distance(&a.get_move_distance(&b).scale(t))
    /*
    let mt = 1.0 - t; // (1 - t)

    let x = mt * mt * s.x + 2.0 * mt * t * c.x + t * t * e.x;
    let y = mt * mt * s.y + 2.0 * mt * t * c.y + t * t * e.y;

    Point { x, y }
    */
}

pub fn quadratic_arc_length(begin: &Point, control: &Point, end: &Point) -> f32 {
    // Vector components
    // v = P1 - P0
    let vx = control.x - begin.x;
    let vy = control.y - begin.y;

    // w = P2 - P1
    let wx = end.x - control.x;
    let wy = end.y - control.y;

    // u = w - v = P2 - 2*P1 + P0
    let ux = wx - vx;
    let uy = wy - vy;

    // Coefficients of the polynomial inside the radical: f(t) = c*t^2 + b*t + a
    let c = ux * ux + uy * uy;
    let b = 2.0 * (ux * vx + uy * vy);
    let a = vx * vx + vy * vy;

    // Handle collinear or degenerate curves (straight line or overlapping points)
    if c.abs() < f32::EPSILON {
        // If c is zero, the path speed is constant: 2 * sqrt(a)
        return 2.0 * a.sqrt();
    }

    // The velocity magnitude is multiplied by 2.0 because P'(t) = 2 * (1-t)(P1-P0) + 2t(P2-P1)
    2.0 * (qd_arc_sup(a, b, c, 1.0) - qd_arc_sup(a, b, c, 0.0))
}

fn qd_arc_sup(a: f32, b: f32, c: f32, t: f32) -> f32 {
    let temp = 2.0 * c * t + b;
    let radical = (c * t * t + b * t + a).sqrt();

    let term1 = (temp * radical) / (4.0 * c);

    let k = 4.0 * a * c - b * b;
    let log_arg = temp + 2.0 * c.sqrt() * radical;

    let term2 = if log_arg > 0.0 {
        (k * log_arg.ln()) / (8.0 * c * c.sqrt())
    } else {
        0.0
    };

    term1 + term2
}

pub fn arc_contains_point(r: f32, p: &Point, begin: &Point, control: &Point, end: &Point) -> bool {
    /*
    let t = match find_t_for_arc(begin, control, end, p) {
        Some(t) => t,
        None => return false,
    };
    */
    let t = closest_t_on_arc(begin, control, end, p);
    let check = compute_arc_point(t, begin, control, end);
    //log(&format!("{check},{p},{r}"));
    inside_circle(p, &check, r)
}

pub fn shift_arc_position(
    a: &Point,
    c: &Point,
    b: &Point,
    r: f32,
    position: &LabelPosition,
) -> [Point; 3] {
    let offset = match position {
        LabelPosition::Center => return [*a, *c, *b],
        LabelPosition::Bottom => R_270,
        LabelPosition::Top => R_90,
    };

    let d1 = a.get_distance_vec(c, r, offset);
    let d2 = c.get_distance_vec(&b, r, offset);

    let vs = a.get_move_distance(c);
    let vc = c.get_move_distance(b);
    let s1 = a.sub_distance(&d1);
    let e1 = b.sub_distance(&d2);
    let c1 = {
        let c1 = c.sub_distance(&d1).add_distance(&vs);
        let c2 = c.sub_distance(&d2);
        force_intersection(&s1, &c1, &c2, &e1.add_distance(&vc))
    };

    [s1, c1, e1]
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

/// Calculates the exact parameter `t` [0.0, 1.0] on a 2D quadratic Bezier curve
/// that minimizes the distance to a target point `m`.
pub fn closest_t_on_arc(begin: &Point, control: &Point, end: &Point, m: &Point) -> f32 {
    // this provides the control point as an inveted vector
    let ax = begin.x - 2.0 * control.x + end.x;
    let ay = begin.y - 2.0 * control.y + end.y;

    // this provides an inveted vector of begin to control
    let bx = 2.0 * (control.x - begin.x);
    let by = 2.0 * (control.y - begin.y);

    // this provides a vector from begin to or test point of m
    let cm_x = begin.x - m.x;
    let cm_y = begin.y - m.y;

    // 2. Compute the derivative coefficients of the squared distance function
    // f(t) = a*t^3 + b*t^2 + c*t + d = 0 (Divided by 2 for optimization)
    let a = 2.0 * (ax * ax + ay * ay);
    let b = 3.0 * (ax * bx + ay * by);
    let c = (bx * bx + by * by) + 2.0 * (ax * cm_x + ay * cm_y);
    let d = bx * cm_x + by * cm_y;

    // start with a t value of 0
    let mut best_t = 0.0;
    // start wtih the largest f32
    let mut min_d_sq = f32::INFINITY;

    for t_sample in [0.0, 0.5, 0.75, 1.0] {
        let pt = compute_arc_point(t_sample, begin, control, end);
        let dx = pt.x - m.x;
        let dy = pt.y - m.y;
        let d_sq = dx * dx + dy * dy;
        // we are looking for the smallest distance squard from our seed point
        // to our check point.
        if d_sq < min_d_sq {
            min_d_sq = d_sq;
            best_t = t_sample;
        }
    }

    // look for our approximte distance  based on best_t
    for _ in 0..5 {
        let a_best_t = a * best_t;

        // Quazi Area of triangle squared and multiply by best_t
        let f_prime_t = (3.0 * a_best_t + 2.0 * b) * best_t + c;
        // Quazi Area of the triangle squared with the c replaced by d and multiplied by best_t
        let f_t = ((a_best_t + b) * best_t + c) * best_t + d;

        // Prevent division-by-zero errors on perfectly flat curves
        if f_prime_t.abs() < f32::EPSILON {
            break;
        }

        let next_t = best_t - f_t / f_prime_t;

        // Clamp to valid parametric boundaries
        best_t = next_t.clamp(0.0, 1.0);
    }

    best_t.abs().clamp(0.0, 1.0)
}
