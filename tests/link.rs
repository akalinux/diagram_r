#![cfg(test)]

mod common;
use approx::assert_relative_eq;
use common::*;
use diagram_r::{Point, bsp::LookupPointResult, constants::ZERO_POINT, diagram::DiagramOpt};
#[test]
fn link_container_update_tests() {
    let mut lc = test_lc_b1_l2();
    let (a, b) = nodes_a_b();
    let mut opt = DiagramOpt::new();
    opt.link_scale = 1.0;
    lc.update(&a, &b, &opt);
    // 2 links act as 4
    let dd = unsafe { lc.draw_data.as_ref().unwrap_unchecked() };
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
    assert_eq!(&res, &LookupPointResult::Bundle((lc.bundles[0].clone(), 0)));
    assert_relative_eq!(lc.get_center(&res).x, center.x, epsilon = 0.001);
    assert_relative_eq!(lc.get_center(&res).y, center.y, epsilon = 0.001);
    res = lc.contains_point(&Point::new(1.0, 0.25));
    assert_eq!(&res, &LookupPointResult::Link((lc.links[0].clone(), 0)));
    left.y = 0.25;
    left.x = 5.0;
    assert_relative_eq!(lc.get_center(&res).x, left.x, epsilon = 0.001);
    assert_relative_eq!(lc.get_center(&res).y, left.y, epsilon = 0.001);

    assert_eq!(lc.contains_point(&ZERO_POINT), LookupPointResult::NoMatch);
}
