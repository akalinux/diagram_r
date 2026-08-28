#![cfg(test)]

mod common;
use approx::assert_relative_eq;
use common::*;
use diagram_r::DiagramOpt;
use diagram_r::constants::HALF;
use diagram_r::link::SubLink;
use diagram_r::link::iters::{ArcIter, FullBoxAccumulate, LineIter, NextPointSet};
use diagram_r::{Point, bsp::LookupPointResult, constants::ZERO_POINT};
use wasm_bindgen_test::wasm_bindgen_test;
#[test]
#[wasm_bindgen_test]
fn link_container_update_tests() {
    let mut opt = DiagramOpt::new();
    opt.link_scale = 1.0;
    let (a, b) = nodes_a_b();
    let lc = test_lc_b1_l2(&a, &b, &opt, 0, (0, 1));
    // 2 links act as 4
    let dd = &lc.draw_data;
    let center = Point::new(5.0, 0.5);
    assert_relative_eq!(dd.bundles[0].x, center.x, epsilon = 0.001);
    assert_relative_eq!(dd.bundles[0].y, center.y, epsilon = 0.001);
    let mut left = Point::new(0.5, 0.25);
    let mut right = Point::new(9.5, 0.25);
    match *&dd.links[0] {
        SubLink::Line([a, b], _) => {
            assert_point!(a, left, 0.001);
            assert_point!(b, right, 0.001);
        }
        _ => panic!("Unexpected SubLink type"),
    }
    left.y = 0.75;
    right.y = 0.75;

    match *&dd.links[1] {
        SubLink::Line([a, b], _) => {
            assert_point!(a, left, 0.001);
            assert_point!(b, right, 0.001);
        }
        _ => panic!("Unexpected SubLink type"),
    }

    let mut res = lc.contains_point(&center);
    assert_eq!(&res, &LookupPointResult::Bundle((0, 0)));
    assert_relative_eq!(lc.get_center(&res).x, center.x, epsilon = 0.001);
    assert_relative_eq!(lc.get_center(&res).y, center.y, epsilon = 0.001);
    res = lc.contains_point(&Point::new(1.0, 0.25));
    assert_eq!(&res, &LookupPointResult::Link((0, 0)));
}

#[test]
fn test_line_iter() {
    let mut iter = LineIter::new(&Point::new(0.0, 3.0), &Point::new(10.0, 3.0), 6.0, 2);
    let mut left = Point::new(0.0, 1.5);
    let mut right = Point::new(10.0, 1.5);
    let mut res = iter.next().unwrap();
    assert_relative_eq!(res.1.x, left.x, epsilon = 0.001);
    assert_relative_eq!(res.3.x, right.x, epsilon = 0.001);
    assert_relative_eq!(res.1.y, left.y, epsilon = 0.001);
    assert_relative_eq!(res.3.y, right.y, epsilon = 0.001);
    left.y = 4.5;
    right.y = 4.5;
    res = iter.next().unwrap();
    assert_relative_eq!(res.1.x, left.x, epsilon = 0.001);
    assert_relative_eq!(res.3.x, right.x, epsilon = 0.001);
    assert_relative_eq!(res.1.y, left.y, epsilon = 0.001);
    assert_relative_eq!(res.3.y, right.y, epsilon = 0.001);
    assert!(iter.next().is_none());

    iter = LineIter::new(&Point::new(10.0, 3.0), &Point::new(0.0, 3.0), 6.0, 1);
    right = Point::new(0.0, 3.0);
    left = Point::new(10.0, 3.0);
    res = iter.next().unwrap();
    assert_relative_eq!(res.1.x, left.x, epsilon = 0.001);
    assert_relative_eq!(res.3.x, right.x, epsilon = 0.001);
    assert_relative_eq!(res.1.y, left.y, epsilon = 0.001);
    assert_relative_eq!(res.3.y, right.y, epsilon = 0.001);
    assert!(iter.next().is_none());
}

#[test]
pub fn point_accumualte_test() {
    let mut a = FullBoxAccumulate::new();
    a.step(&Point { x: 1.0, y: 1.0 });
    a.step(&Point { x: 2.0, y: 1.0 });
    a.step(&Point { x: 1.5, y: 2.0 });
    a.step(&Point { x: 1.5, y: 2.0 });
    assert_eq!(a.full_box_from(), (1.0, 2.0, 1.0, 2.0))
}

#[test]
pub fn next_point_set_tests() {
    let p = NextPointSet::new(&ZERO_POINT, &Point { x: 0.0, y: 1.0 }, 5.0, 0.25, HALF, 0.0);
    assert_point!(p.point(0.0), Point::new(0.0, -2.5), 0.001);
    assert_point!(p.point(1.0), Point::new(0.0, 2.5), 0.001);
}

#[test]
pub fn arc_point_test() {
    let a = ZERO_POINT;
    let b = Point::new(5.0, 5.0);
    let c = Point::new(10.0, 0.0);
    let mut iter = ArcIter::new(&a, &b, &c, 5.0, 1);
    let (_, w, set, y) = iter.next().unwrap();
    let (_, x) = set.unwrap();
    assert_point!(w, a, 0.001);
    assert_point!(x, b, 0.001);
    assert_point!(y, c, 0.001);
    assert!(iter.next().is_none());
}
