use crate::{
    Point, Transform,
    constants::{RAD2DEG, TRIANGLE_MARGINE_FOR_ERROR},
    square::Corners,
};

pub type AngleNorthSouth = (f64, f64, f64);
pub type FullBox = (Point, Point, Point, Point);
pub fn to_map_xy(p: &Point, t: &Transform) -> Point {
    let px = p.x - t.x;
    let py = p.y - t.y;
    let x = px / t.k;
    let y = py / t.k;
    Point { x, y }
}

pub fn get_xy(cx: f64, cy: f64, r: f64, degree: f64) -> Point {
    let rad = degree.to_radians();

    let x = cx + r * rad.cos();
    let y = cy + r * rad.sin();
    Point { x, y }
}

pub fn get_angle(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x1 - x2;
    let dy = y1 - y2;

    let base = dy.atan2(dx) * RAD2DEG; //- 180;
    if base < 0.0 {
        return base + 360.0;
    }

    base
}

pub fn triangle_area(x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> f64 {
    return (x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2)).abs() * 0.5;
}

/// Provides the [Corners] from the given 4 [Point] instances.
pub fn compute_line_box(ne: &Point, points: [&Point; 3]) -> Corners {
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

pub fn full_box_from(a: &Point, b: &Point, r: f64) -> (FullBox, AngleNorthSouth) {
    let angle = get_angle(a.x, a.y, b.x, b.y);
    let north = angle + 90.0;
    let south = north + 180.0;
    let nw = get_xy(a.x, a.y, r, north);
    let ne = get_xy(b.x, b.y, r, north);
    let sw = get_xy(a.x, a.y, r, south);
    let se = get_xy(b.x, b.y, r, south);
    ((nw, ne, sw, se), (angle, north, south))
}

pub fn inside_box(pbox: &FullBox, p: &Point) -> bool {
    let (nw, ne, sw, se) = pbox;
    let box_area = (triangle_area(ne.x, ne.y, nw.x, nw.y, se.x, se.y)
        + triangle_area(ne.x, ne.y, nw.x, nw.y, sw.x, sw.y))
        * TRIANGLE_MARGINE_FOR_ERROR;

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

pub fn get_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    return get_distance_square(x1, y1, x2, y2).sqrt();
}

pub fn get_distance_square(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    return (x1 - x2).powi(2) + (y1 - y2).powi(2);
}

pub fn compute_r_for_even_space_on_circle(r: f64, points: f64) -> f64 {
    let degree = 360.0 / points;
    let a = get_xy(0.0, 0.0, r, 0.0);
    let b = get_xy(0.0, 0.0, r, degree);
    let cmp = get_distance(a.x, a.y, b.x, b.y);
    let scale = r / cmp;
    return r * scale;
}

pub fn inside_circle(p: &Point, c: &Point, r: f64) -> bool {
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
