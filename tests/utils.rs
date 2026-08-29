#![cfg(test)]

mod common;
use approx::assert_relative_eq;
use diagram_r::{
    Point,
    constants::{R_270, ZERO_POINT},
    utils::{compute_arc_point, full_box_from, get_intersection, inside_arc, inside_box},
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
fn inside_arc_test() {
    let s = ZERO_POINT;
    let c = Point::new(5.0, 5.0);
    let e = Point::new(0.0, 10.0);
    assert!(inside_arc(&s, &c, &e, &Point { x: 1.0, y: 5.5 }));
    assert!(!inside_arc(&s, &c, &e, &Point { x: 10.0, y: 10.0 }));
    let c = Point::new(0.0, 5.0);
    assert!(inside_arc(&s, &c, &e, &Point { x: 0.0, y: 5.0 }));
    assert!(!inside_arc(&s, &c, &e, &Point { x: 0.0, y: 50.0 }));
}
