#![cfg(test)]

mod common;
use approx::assert_relative_eq;
use diagram_r::{
    Point,
    constants::{R_270, ZERO_POINT},
    utils::{
        arc_contains_point, closest_t_on_arc, closest_t_on_arc2, compute_arc_point, full_box_from,
        get_intersection, inside_box, side_of_line,
    },
};
use wasm_bindgen_test::wasm_bindgen_test;

use crate::common::full_testbox2;

#[test]
#[wasm_bindgen_test]
fn full_box_from_testx() {
    let a = Point::new(0.0, 2.5);
    let b = Point::new(10.0, 2.5);
    let r = 2.5;
    let ((nw, ne, sw, se), (distance, rad, _)) = full_box_from(&a, &b, r);
    assert_relative_eq!(rad, R_270, epsilon = 0.001);
    assert_relative_eq!(distance.y, 2.5, epsilon = 0.001);
    assert_relative_eq!(distance.x, 0.0, epsilon = 0.001);
    for (new, ctrl) in [
        (nw, ZERO_POINT),
        (ne, Point::new(10.0, 0.0)),
        (sw, Point::new(0.0, 5.0)),
        (se, Point::new(10.0, 5.0)),
    ] {
        assert_relative_eq!(new.x, ctrl.x, epsilon = 0.001);
        assert_relative_eq!(new.y, ctrl.y, epsilon = 0.001);
    }
}

#[test]
#[wasm_bindgen_test]
fn inside_box_tests() {
    let (pbox, _) = full_testbox2();
    // top left
    assert!(inside_box(&pbox, &ZERO_POINT));
    // center
    assert!(inside_box(&pbox, &Point::new(5.0, 2.5)));
    // outside
    assert!(!inside_box(&pbox, &Point::new(15.0, 2.5)));
}

#[test]
fn test_point_arc_move() {
    let dst = Point::new(10.0, 0.0);
    let p = dst.get_distance_vec(&ZERO_POINT, 5.0, 0.0);
    assert_point!(p, Point::new(5.0, 0.0), 0.001);
}

#[test]
fn validate_intersect_test() {
    let a = ZERO_POINT;
    let b = Point::new(5.0, 5.0);
    let c = Point::new(0.0, 5.0);
    let d = Point::new(5.0, 0.0);
    let p = get_intersection(&a, &b, &c, &d).unwrap();
    assert_point!(p, Point::new(2.5, 2.5), 0.001);
    let p = get_intersection(&a, &d, &c, &b);
    assert!(p.is_none())
}

#[test]
fn arc_point_test() {
    let s = ZERO_POINT;
    let c = Point::new(5.0, 5.0);
    let e = Point::new(0.0, 10.0);
    let p = compute_arc_point(0.5, &s, &c, &e);
    assert_point!(p, Point { y: 5.0, x: 2.5 }, 0.001);
}

#[test]
fn arc_contains_point_test() {
    let s = ZERO_POINT;
    let c = Point::new(5.0, 5.0);
    let e = Point::new(0.0, 10.0);

    for i in [0.25, 0.5, 0.75] {
        let p = compute_arc_point(i, &s, &c, &e);
        assert!(arc_contains_point(0.5, &p, &s, &c, &e))
    }
}

#[test]
fn get_t_from_p_test() {
    let s = ZERO_POINT;
    let c = Point::new(5.0, 5.0);
    let e = Point::new(0.0, 10.0);

    for i in [0.15, 0.25, 0.33, 0.45, 0.5, 0.55, 0.66, 0.75, 0.85, 0.90] {
        //for i in [0.15, 0.25, 0.33, 0.45, 0.5] {
        let p = compute_arc_point(i, &s, &c, &e);
        //println!("Real Point: {p}, Raw: {i}");
        let cmp = closest_t_on_arc(&s, &c, &e, &p);
        assert_relative_eq!(cmp, i, epsilon = 0.001);
        let cmp2 = closest_t_on_arc2(&s, &c, &e, &p);
        println!("  Slow: {cmp} Fast: {cmp2}");

        //let [x, y, z] = shift_arc_position(&s, &c, &e, 1.25, &diagram_r::LabelPosition::Top);
        //let p = compute_arc_point(i, &x, &y, &z);
        //let cmp = closest_t_on_arc(&s, &c, &e, &p);
        //let cmp2 = closest_t_on_arc2(&s, &c, &e, &p);
        //let p2 = compute_arc_point(cmp, &x, &y, &z);
        //let center = s.get_center(&e);

        //println!(
        //            "     Slow: {cmp} Fast: {cmp2}, Expected Angle: {}\n",
        //          p.get_degree(&center) * HALF
        //    );
        //println!("{i},{cmp2}");
        //break;
    }
}

#[test]
fn line_side_test() {
    assert_eq!(side_of_line(&ZERO_POINT, &ZERO_POINT, &ZERO_POINT), 0.0);

    // above the line
    assert!(
        side_of_line(
            &ZERO_POINT,
            &Point { x: 10.0, y: 0.0 },
            &Point { x: 5.0, y: 5.0 }
        ) < 0.0,
    );
    // below the line
    assert!(
        side_of_line(
            &ZERO_POINT,
            &Point { x: 10.0, y: 0.0 },
            &Point { x: 5.0, y: -5.0 }
        ) > 0.0,
    );
}

#[test]
fn center_angle_from() {
    assert_relative_eq!(
        Point::new(5.0, 5.0)
            .center_radian_to(&ZERO_POINT, &Point::new(10.0, 0.0))
            .to_degrees(),
        90.0,
        epsilon = 0.001,
    );
}
