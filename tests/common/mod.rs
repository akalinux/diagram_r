#![allow(dead_code)]
use diagram_r::{
    DiagramOpt, Point,
    link::{Animation, Bundle, Link, LinkContainer, LinkSet},
    node::Node,
    square::Square,
    utils::{FullBox, full_box_from},
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
        Square::new(0.0, 0.0, 10.0, 2.0),
        String::from("box-a"),
        0,
        vec![0],
    )
}
pub fn nodes_a_b() -> (Node, Node) {
    (
        Node::new(
            Square::new(0.0, 0.0, 1.0, 1.0),
            String::from("node-a"),
            0,
            vec![0],
        ),
        Node::new(
            Square::new(9.0, 0.0, 1.0, 1.0),
            String::from("node-b"),
            0,
            vec![0],
        ),
    )
}

pub fn full_testbox2() -> (FullBox, (Point, f64)) {
    let a = Point::new(0.0, 2.5);
    let b = Point::new(10.0, 2.5);
    let r = 2.5;
    full_box_from(&a, &b, r)
}

pub fn data_lc_b1_l2() -> (Link, Link, Bundle) {
    let c = Bundle::new(1, String::from("test bundle"), vec![0, 1]);
    let a = Link::new(1, String::from("link a"), Animation::ToDst);
    let b = Link::new(2, String::from("link b"), Animation::ToDst);
    (a, b, c)
}

pub fn default_link_set(ids: (usize, usize)) -> LinkSet {
    let (a, b, c) = data_lc_b1_l2();
    LinkSet::new(vec![a, b], vec![c], ids.0, ids.1)
}
pub fn test_lc_b1_l2(
    src: &Node,
    dst: &Node,
    opt: &DiagramOpt,
    id: usize,
    ids: (usize, usize),
) -> LinkContainer {
    LinkContainer::new(default_link_set(ids), src, dst, opt, id)
}

pub fn test_lc_b1_l3(
    src: &Node,
    dst: &Node,
    opt: &DiagramOpt,
    id: usize,
    ids: (usize, usize),
) -> LinkContainer {
    let (a, b, c) = data_lc_b1_l2();
    let d = Link::new(2, String::from("link b"), Animation::ToDst);
    let ls = LinkSet::new(vec![a, b, d], vec![c], ids.0, ids.1);
    LinkContainer::new(ls, src, dst, opt, id)
}
