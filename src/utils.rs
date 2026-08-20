use js_sys::Number;

use crate::{Point, Transform, constants::AREA_SCALE_EPSILON, square::Corners};

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

pub fn get_xy(cx: f32, cy: f32, r: f32, degree: f32) -> Point {
    let rad = degree.to_radians();

    let x = cx + r * rad.cos();
    let y = cy + r * rad.sin();
    Point { x, y }
}

pub fn get_angle(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x1 - x2;
    let dy = y1 - y2;

    //let base = dy.atan2(dx) * RAD2DEG; //- 180;
    let base = dy.atan2(dx).to_degrees();
    if base < 0.0 {
        return base + 360.0;
    }

    base
}

pub fn triangle_area(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> f32 {
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

pub fn north_box_from(a: &Point, b: &Point, r: f32) -> (Point, Point, Point, f32) {
    let angle = get_angle(a.x, a.y, b.x, b.y);
    let north = angle + 90.0;
    let nw = get_xy(a.x, a.y, r, north);
    let distance = nw.get_move_distance(&a);

    let ne = b.sub_distance(&distance);
    (nw, ne, distance, north)
}
pub fn full_box_from(a: &Point, b: &Point, r: f32) -> (FullBox, (Point, f32)) {
    let (nw, ne, distance, north) = north_box_from(a, b, r);

    let sw = a.add_distance(&distance);
    let se = b.add_distance(&distance);
    ((nw, ne, sw, se), (distance, north))
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
    let degree = 360.0 / points;
    let a = get_xy(0.0, 0.0, r, 0.0);
    let b = get_xy(0.0, 0.0, r, degree);
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
