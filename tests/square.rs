#![cfg(test)]

use diagram_r::{Point, constants::ZERO_POINT, square::Square};

mod common;
use common::*;
use wasm_bindgen_test::wasm_bindgen_test;
#[test]
#[wasm_bindgen_test]
fn get_center() {
    let a = square_a();
    assert_eq!(a.get_center(), Point::new(2.5, 2.5));
}

#[test]
#[wasm_bindgen_test]
fn index_tests() {
    assert_eq!(square_a().idx(5), (0..=5, 0..=5, 25.0));
    assert_eq!(square_b().idx(5), (0..=10, 0..=10, 100.0));
    assert_eq!(square_c().idx(5), (-5..=5, 0..=10, 100.0));
    assert_eq!(square_d().idx(5), (0..=10, -5..=5, 100.0));
}

#[test]
#[wasm_bindgen_test]
fn from_min_max() {
    let a = Square::from((1.0, 2.0, 2.0, 3.0));
    assert_eq!(a, Square::new(1.0, 2.0, 1.0, 1.0));
}

#[test]
#[wasm_bindgen_test]
fn screen_center() {
    let a = Square {
        width: 5.0,
        height: 5.0,
        x: 0.0,
        y: 0.0,
    };
    let b = Square {
        width: 10.0,
        height: 10.0,
        x: 0.0,
        y: 0.0,
    };
    assert_eq!(a.center(&b), Point { x: 2.5, y: 2.5 });
    assert_eq!(b.center(&a), Point { x: 5.0, y: 5.0 });
    let c = Square {
        width: 10.0,
        height: 10.0,
        x: -5.0,
        y: -5.0,
    };
    assert_eq!(c.get_center(), Point { x: 0.0, y: 0.0 });
    assert_eq!(c.center(&b), Point { x: 5.0, y: 5.0 });
    assert_eq!(c.center(&c), Point { x: 5.0, y: 5.0 });
    assert_eq!(b.center(&c), Point { x: 10.0, y: 10.0 });
}

#[test]
#[wasm_bindgen_test]
fn contains_point() {
    let s = Square::new(0.0, 0.0, 2.0, 2.0);
    assert!(s.contains_point(&ZERO_POINT));
    let max = Point::new(s.max_x(), s.max_y());

    assert!(s.contains_point(&max));
    assert!(!s.contains_point(&Point::new(3.0, 3.0)));
    let s = Square {
        x: 4.575,
        y: 0.0749999999999989,
        width: 0.85,
        height: 0.85,
    };
    let p = Point { x: 5.0, y: 0.0 };

    assert!(!s.contains_point(&p));
}

#[test]
#[wasm_bindgen_test]
fn distance_tests() {
    let mut p = square_a();
    let m = Point::new(2.0, 2.0);
    assert_eq!(p.get_distance(&m), m);
    p.move_distance(&m);
    assert_eq!(p.x, 2.0);
    assert_eq!(p.y, 2.0);
}
