#![cfg(test)]

use std::{cell::RefCell, rc::Rc};

use diagram_r::{
    Point,
    bsp::LookupPointResult,
    constants::ZERO_POINT,
    diagram::{DiagramCore, DiagramOpt},
};

use crate::common::{box_a, data_lc_b1_l2, nodes_a_b};
mod common;

pub fn base_diagram() -> Rc<RefCell<DiagramCore>> {
    let ct = DiagramCore::new(DiagramOpt::new());

    let (node_a, node_b) = nodes_a_b();
    let box_a = box_a();
    let (link_a, link_b, bundle) = data_lc_b1_l2();

    match ct.borrow_mut().set_data(
        vec![box_a],
        vec![node_a, node_b],
        vec![link_a, link_b],
        vec![bundle],
    ) {
        Err(_) => panic!("Setting data failed!"),
        Ok(_) => (),
    };

    ct
}

#[test]
fn buld_ok_test() {
    DiagramCore::new(DiagramOpt::new());
}

#[test]
fn set_data_test() {
    base_diagram();
}

#[test]
fn in_point_tests() {
    let diagram = base_diagram();

    let (node_a, node_b) = nodes_a_b();
    assert_eq!(
        diagram
            .borrow()
            .idx
            .contains_point(&ZERO_POINT, &*diagram.borrow()),
        LookupPointResult::Node(node_a.id)
    );
    assert_eq!(
        diagram
            .borrow()
            .idx
            .contains_point(&Point { x: 10.0, y: 0.0 }, &*diagram.borrow()),
        LookupPointResult::Node(node_b.id)
    );
    let box_a = box_a();
    assert_eq!(
        diagram
            .borrow()
            .idx
            .contains_point(&Point { x: 5.0, y: 0.0 }, &*diagram.borrow()),
        LookupPointResult::Box(box_a.id)
    );
    let (link_a, link_b, bundle) = data_lc_b1_l2();
    assert_eq!(
        diagram
            .borrow()
            .idx
            .contains_point(&Point { x: 5.0, y: 0.5 }, &*diagram.borrow()),
        LookupPointResult::Bundle((bundle.clone(), 0))
    );
}
