#![allow(dead_code)]
use diagram_r::{
    Point,
    link::{Animation, Bundle, Link, LinkContainer},
    node::Node,
    square::Square,
    utils::{AngleNorthSouth, FullBox, full_box_from},
};
pub fn square_a() -> Square {
    Square::new(0.0, 0.0, 5.0, 5.0)
}
pub fn square_b() -> Square {
    Square::new(0.0, 0.0, 10.0, 10.0)
}
pub fn square_c() -> Square {
    Square::new(-1.0, 0.0, 10.0, 10.0)
}
pub fn square_d() -> Square {
    Square::new(0.0, -1.0, 10.0, 10.0)
}

pub fn box_a() -> Node {
    Node::new(
        2,
        Square::new(0.0, 0.0, 10.0, 2.0),
        String::from("box-a"),
        0,
        vec![0],
    )
}
pub fn nodes_a_b() -> (Node, Node) {
    (
        Node::new(
            0,
            Square::new(0.0, 0.0, 1.0, 1.0),
            String::from("node-a"),
            0,
            vec![0],
        ),
        Node::new(
            1,
            Square::new(9.0, 0.0, 1.0, 1.0),
            String::from("node-b"),
            0,
            vec![0],
        ),
    )
}
pub fn full_testbox() -> (FullBox, AngleNorthSouth) {
    let a = Point::new(0.0, 2.5);
    let b = Point::new(10.0, 2.5);
    let r = 2.5;
    full_box_from(&a, &b, r)
}

pub fn data_lc_b1_l2() -> (Link, Link, Bundle) {
    let c = Bundle::new(0, 1, 0, String::from("test bundle"), vec![0, 1]);
    let a = Link::new(0, 1, 0, String::from("link a"), Animation::ToDst);
    let b = Link::new(1, 0, 0, String::from("link b"), Animation::ToDst);
    (a, b, c)
}
pub fn test_lc_b1_l2() -> LinkContainer {
    let (a, b, c) = data_lc_b1_l2();
    let mut res = LinkContainer::new(b.link_id());
    res.add_link(a);
    res.add_link(b);
    let _ = res.add_bundle(c);

    res
}
