#![cfg(test)]

mod common;
use approx::{assert_relative_eq, assert_relative_ne};
use diagram_r::{
    Point,
    constants::ZERO_POINT,
    utils::{full_box_from, get_angle, inside_box, invert_dst},
};
use wasm_bindgen_test::wasm_bindgen_test;

use crate::common::full_testbox2;

#[wasm_bindgen_test]
#[test]
fn angle_test() {
    let start = ZERO_POINT;
    let end = Point::new(5.0, 0.0);
    let angle = get_angle(start.x, start.y, end.x, end.y);
    assert_relative_eq!(angle, 180.0, epsilon = 0.001);
}

#[test]
#[wasm_bindgen_test]
fn full_box_from_testx() {
    let a = Point::new(0.0, 2.5);
    let b = Point::new(10.0, 2.5);
    let r = 2.5;
    let ((nw, ne, sw, se), (distance, angle, _)) = full_box_from(&a, &b, r);
    assert_relative_eq!(angle, 270.0, epsilon = 0.001);
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
fn invert_dst_test() {
    let (p, _) = invert_dst(&Point { x: 0.0, y: 5.0 }, &Point { x: 5.0, y: 5.0 }, 5.0);
    assert_relative_eq!(p.x, 0.0, epsilon = 0.001);
    assert_relative_eq!(p.y, 5.0, epsilon = 0.001);
}
