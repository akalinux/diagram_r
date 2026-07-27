#![cfg(test)]

mod common;
use approx::assert_relative_eq;
use common::full_testbox;
use diagram_r::{
    Point,
    constants::ZERO_POINT,
    utils::{get_angle, inside_box},
};
#[test]
fn angle_test() {
    let start = ZERO_POINT;
    let end = Point::new(5.0, 0.0);
    let angle = get_angle(start.x, start.y, end.x, end.y);
    assert_relative_eq!(angle, 180.0, epsilon = 0.001);
}

#[test]
fn full_box_from_tests() {
    let ((nw, ne, sw, se), (angle, north, south)) = full_testbox();
    assert_relative_eq!(angle, 180.0, epsilon = 0.001);
    assert_relative_eq!(north, 270.0, epsilon = 0.001);
    assert_relative_eq!(south, 450.0, epsilon = 0.001);
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
fn inside_box_tests() {
    let (pbox, _) = full_testbox();
    // top left
    assert!(inside_box(&pbox, &ZERO_POINT));
    // center
    assert!(inside_box(&pbox, &Point::new(5.0, 2.5)));
    // outside
    assert!(!inside_box(&pbox, &Point::new(15.0, 2.5)));
}
