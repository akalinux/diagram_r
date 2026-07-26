#![cfg(test)]

use approx::assert_relative_eq;
use diagram_r::{Point, constants::ZERO_POINT, utils::get_angle};
#[test]
fn angle_test() {
    let start = ZERO_POINT;
    let end = Point::new(5.0, 0.0);
    let angle = get_angle(start.x, start.y, end.x, end.y);
    assert_relative_eq!(angle, 180.0, epsilon = 0.001);
}
