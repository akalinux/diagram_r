#![cfg(test)]

use diagram_r::{Point, square::Square};

fn square_a() -> Square {
    Square::new(0.0, 0.0, 5.0, 5.0)
}
fn square_b() -> Square {
    Square::new(0.0, 0.0, 10.0, 10.0)
}
fn square_c() -> Square {
    Square::new(-1.0, 0.0, 10.0, 10.0)
}
fn square_d() -> Square {
    Square::new(0.0, -1.0, 10.0, 10.0)
}
#[test]
fn get_center() {
    let a = square_a();
    assert_eq!(a.get_center(), Point::new(2.5, 2.5));
}

#[test]
fn index_tests() {
    assert_eq!(square_a().idx(5), (0..=5, 0..=5));
    assert_eq!(square_b().idx(5), (0..=10, 0..=10));
    assert_eq!(square_c().idx(5), (-5..=5, 0..=10));
    assert_eq!(square_d().idx(5), (0..=10, -5..=5));
}

#[test]
fn from_min_max() {
    let a = Square::from((1.0, 2.0, 2.0, 3.0));
    assert_eq!(a, Square::new(1.0, 2.0, 1.0, 1.0));
}

#[test]
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
