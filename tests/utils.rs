#![cfg(test)]

mod common;
use approx::assert_relative_eq;
use diagram_r::{
    Point,
    constants::ZERO_POINT,
    utils::{compute_bunlde_points, full_box_from, get_angle, inside_box},
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
    let ((nw, ne, sw, se), (distance, angle)) = full_box_from(&a, &b, r);
    assert_relative_eq!(angle, 180.0, epsilon = 0.001);
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
#[wasm_bindgen_test]
fn compute_bundle_points_tests() {
    assert_eq!(
        compute_bunlde_points(&ZERO_POINT, &Point::new(3.0, 3.0), 0),
        vec![]
    );

    let mut res = compute_bunlde_points(&ZERO_POINT, &Point::new(3.0, 3.0), 1);
    assert_relative_eq!(res[0].x, 1.5, epsilon = 0.0001);
    assert_relative_eq!(res[0].y, 1.5, epsilon = 0.0001);
    res = compute_bunlde_points(&ZERO_POINT, &Point::new(4.0, 4.0), 2);
    assert_relative_eq!(res[0].x, 1.0, epsilon = 0.0001);
    assert_relative_eq!(res[0].y, 1.0, epsilon = 0.0001);
    assert_relative_eq!(res[1].x, 3.0, epsilon = 0.0001);
    assert_relative_eq!(res[1].y, 3.0, epsilon = 0.0001);
    res = compute_bunlde_points(&ZERO_POINT, &Point::new(6.0, 6.0), 3);
    assert_relative_eq!(res[0].x, 1.0, epsilon = 0.0001);
    assert_relative_eq!(res[0].y, 1.0, epsilon = 0.0001);
    assert_relative_eq!(res[1].x, 3.0, epsilon = 0.0001);
    assert_relative_eq!(res[1].y, 3.0, epsilon = 0.0001);
    assert_relative_eq!(res[2].x, 5.0, epsilon = 0.0001);
    assert_relative_eq!(res[2].y, 5.0, epsilon = 0.0001);
}
