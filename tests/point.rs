#![cfg(test)]

mod common;

use approx::assert_relative_eq;
use diagram_r::{Point, constants::ZERO_POINT};

#[test]
pub fn get_point_distance_test() {
    let d = ZERO_POINT.get_distance_vec(&Point { x: 0.0, y: 1.0 }, 5.0, 0.0);
    assert_relative_eq!(d.distance(&ZERO_POINT), 5.0, epsilon = 0.001);
}
