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

fn test_points(diagram: Rc<RefCell<DiagramCore>>, p: Point) {
    let (node_a, node_b) = nodes_a_b();

    assert_eq!(
        diagram
            .borrow()
            .idx
            .contains_point(&(ZERO_POINT.move_distance(&p)), &*diagram.borrow()),
        LookupPointResult::Node(node_a.id)
    );

    assert_eq!(
        diagram.borrow().idx.contains_point(
            &Point { x: 10.0, y: 0.000 }.move_distance(&p),
            &*diagram.borrow()
        ),
        LookupPointResult::Node(node_b.id)
    );
    let box_a = box_a();
    assert_eq!(
        diagram.borrow().idx.contains_point(
            &Point { x: 5.0, y: 0.0 }.move_distance(&p),
            &*diagram.borrow()
        ),
        LookupPointResult::Box(box_a.id)
    );
    let (link_a, link_b, bundle) = data_lc_b1_l2();
    assert_eq!(
        diagram.borrow().idx.contains_point(
            &Point { x: 5.0, y: 0.5 }.move_distance(&p),
            &*diagram.borrow()
        ),
        LookupPointResult::Bundle((bundle.clone(), 0))
    );
    assert_eq!(
        diagram.borrow().idx.contains_point(
            &Point { x: 2.5, y: 0.21 }.move_distance(&p),
            &*diagram.borrow()
        ),
        LookupPointResult::Link((link_a.clone(), 0))
    );

    assert_eq!(
        diagram.borrow().idx.contains_point(
            &Point { x: 2.5, y: 0.66 }.move_distance(&p),
            &*diagram.borrow()
        ),
        LookupPointResult::Link((link_b.clone(), 1))
    );
}
#[test]
fn in_point_tests() {
    let diagram = base_diagram();
    test_points(diagram, ZERO_POINT);
}

fn reload_data() -> Rc<RefCell<DiagramCore>> {
    let diagram = base_diagram();

    let (node_a, node_b) = nodes_a_b();
    let box_a = box_a();
    let (link_a, link_b, bundle) = data_lc_b1_l2();

    match diagram.borrow_mut().set_data(
        vec![box_a],
        vec![node_a, node_b],
        vec![link_a, link_b],
        vec![bundle],
    ) {
        Err(_) => panic!("Setting data failed!"),
        Ok(_) => (),
    };
    diagram
}
#[test]
fn reload_data_test() {
    reload_data();
}

#[test]
fn test_reloaded_points() {
    let diagram = reload_data();
    test_points(diagram, ZERO_POINT);
}

#[test]
fn test_move_box() {
    let diagram = reload_data();
    test_points(Rc::clone(&diagram), ZERO_POINT);
    let distance = &Point { x: 5.0, y: 5.0 };
    println!("{:?}", unsafe {
        diagram
            .borrow()
            .nodes
            .get(&2)
            .unwrap_unchecked()
            .get()
            .clone()
            .layout
    });
    diagram.borrow_mut().move_nodes(distance, &[0, 1, 2]);

    test_points(diagram, *distance);
}
