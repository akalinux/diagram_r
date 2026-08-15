#![cfg(test)]

mod common;
use approx::assert_relative_eq;
use common::*;
use diagram_r::DiagramOpt;
use diagram_r::link::{Animation, Link, compute_animation};
use diagram_r::{Point, bsp::LookupPointResult, constants::ZERO_POINT};
use wasm_bindgen_test::wasm_bindgen_test;
#[test]
#[wasm_bindgen_test]
fn link_container_update_tests() {
    let mut opt = DiagramOpt::new();
    opt.link_scale = 1.0;
    let (a, b) = nodes_a_b();
    let mut lc = test_lc_b1_l2(&a, &b, &opt, 0, (0, 1));
    // 2 links act as 4
    let dd = &lc.draw_data;
    let center = Point::new(5.0, 0.5);
    assert_relative_eq!(dd.bundles[0].x, center.x, epsilon = 0.001);
    assert_relative_eq!(dd.bundles[0].y, center.y, epsilon = 0.001);
    let mut left = Point::new(0.5, 0.25);
    let mut right = Point::new(9.5, 0.25);
    assert_relative_eq!(dd.links[0].0.x, left.x, epsilon = 0.001);
    assert_relative_eq!(dd.links[0].1.x, right.x, epsilon = 0.001);
    assert_relative_eq!(dd.links[0].0.y, left.y, epsilon = 0.001);
    assert_relative_eq!(dd.links[0].1.y, right.y, epsilon = 0.001);
    left.y = 0.75;
    right.y = 0.75;
    assert_relative_eq!(dd.links[1].0.x, left.x, epsilon = 0.001);
    assert_relative_eq!(dd.links[1].1.x, right.x, epsilon = 0.001);
    assert_relative_eq!(dd.links[1].0.y, left.y, epsilon = 0.001);
    assert_relative_eq!(dd.links[1].1.y, right.y, epsilon = 0.001);

    let cmp = lc.get_center(&LookupPointResult::NoMatch);
    assert_relative_eq!(cmp.x, center.x, epsilon = 0.001);
    assert_relative_eq!(cmp.y, center.y, epsilon = 0.001);
    let mut res = lc.contains_point(&center);
    assert_eq!(&res, &LookupPointResult::Bundle((0, 0)));
    assert_relative_eq!(lc.get_center(&res).x, center.x, epsilon = 0.001);
    assert_relative_eq!(lc.get_center(&res).y, center.y, epsilon = 0.001);
    res = lc.contains_point(&Point::new(1.0, 0.25));
    assert_eq!(&res, &LookupPointResult::Link((0, 0)));
    left.y = 0.25;
    left.x = 5.0;
    assert_relative_eq!(lc.get_center(&res).x, left.x, epsilon = 0.001);
    assert_relative_eq!(lc.get_center(&res).y, left.y, epsilon = 0.001);

    assert_eq!(lc.contains_point(&ZERO_POINT), LookupPointResult::NoMatch);
    lc = test_lc_b1_l3(&a, &b, &opt, 0, (0, 1));
    assert_eq!(
        lc.draw_data.links[1],
        (a.layout.get_center(), b.layout.get_center())
    );
}

#[test]
#[wasm_bindgen_test]
fn animation_tests() {
    let mut link = Link::new(0, String::from("testing"), Animation::Both);
    let mut a = Vec::new();
    let (src, dst) = (Point::new(0.0, 2.5), Point::new(10.0, 2.5));
    compute_animation(&link, &(src, dst), &mut a, 5.0, 270.0);
    assert_relative_eq!(a[0].0.x, 0.0, epsilon = 0.0001);
    assert_relative_eq!(a[0].1.x, 10.0, epsilon = 0.0001);
    assert_relative_eq!(a[0].0.y, 1.25, epsilon = 0.0001);
    assert_relative_eq!(a[0].1.y, 1.25, epsilon = 0.0001);
    assert_relative_eq!(a[1].0.x, 0.0, epsilon = 0.0001);
    assert_relative_eq!(a[1].1.x, 10.0, epsilon = 0.0001);
    assert_relative_eq!(a[1].0.y, 3.75, epsilon = 0.0001);
    assert_relative_eq!(a[1].1.y, 3.75, epsilon = 0.0001);
    assert_relative_eq!(a[0].2, 5.0 / 3.0, epsilon = 0.0001);
    assert_relative_eq!(a[1].2, 5.0 / 3.0, epsilon = 0.0001);
    link.animation = Animation::ToDst;
    a.clear();
    compute_animation(&link, &(src, dst), &mut a, 5.0, 270.0);
    assert_eq!(a[0].0, src);
    assert_eq!(a[0].1, dst);
}
