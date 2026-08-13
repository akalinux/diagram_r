#![cfg(test)]

use std::{cell::RefCell, rc::Rc};

use diagram_r::{
    Point,
    bsp::LookupPointResult,
    constants::ZERO_POINT,
    diagram::{DiagramCore, DiagramOpt},
};
use wasm_bindgen_test::wasm_bindgen_test;

use crate::common::{box_a, data_lc_b1_l2, default_link_set, nodes_a_b};
mod common;

pub fn base_diagram() -> Rc<RefCell<DiagramCore>> {
    let mut ops = DiagramOpt::new();
    ops.index_step = 1;
    let ct = DiagramCore::new(ops);

    let (node_a, node_b) = nodes_a_b();
    let links = vec![default_link_set((1, 2))];

    match ct
        .borrow_mut()
        .set_data(vec![box_a()], vec![node_a, node_b], links)
    {
        Err(_) => panic!("Setting data failed!"),
        Ok(_) => (),
    };

    ct
}

#[test]
#[wasm_bindgen_test]
fn buld_ok_test() {
    let mut ops = DiagramOpt::new();
    ops.index_step = 1;
    DiagramCore::new(ops);
}

#[test]
#[wasm_bindgen_test]
fn set_data_test() {
    base_diagram();
}

fn test_points(diagram: Rc<RefCell<DiagramCore>>, p: Point) {
    assert_eq!(
        diagram
            .borrow()
            .idx
            .contains_point(&(ZERO_POINT.add_distance(&p)), &*diagram.borrow()),
        LookupPointResult::Node(1)
    );

    assert_eq!(
        diagram.borrow().idx.contains_point(
            &Point { x: 10.0, y: 0.000 }.add_distance(&p),
            &*diagram.borrow()
        ),
        LookupPointResult::Node(2)
    );
    assert_eq!(
        diagram.borrow().idx.contains_point(
            &Point { x: 5.0, y: 0.0 }.add_distance(&p),
            &*diagram.borrow()
        ),
        LookupPointResult::Box(0)
    );
    let (link_a, link_b, bundle) = data_lc_b1_l2();
    assert_eq!(
        diagram.borrow().idx.contains_point(
            &Point { x: 5.0, y: 0.5 }.add_distance(&p),
            &*diagram.borrow()
        ),
        LookupPointResult::Bundle((bundle.clone(), 0, 0))
    );
    assert_eq!(
        diagram.borrow().idx.contains_point(
            &Point { x: 2.5, y: 0.21 }.add_distance(&p),
            &*diagram.borrow()
        ),
        LookupPointResult::Link((link_a.clone(), 0, 0))
    );

    assert_eq!(
        diagram.borrow().idx.contains_point(
            &Point { x: 2.5, y: 0.66 }.add_distance(&p),
            &*diagram.borrow()
        ),
        LookupPointResult::Link((link_b.clone(), 1, 0))
    );
}
#[test]
#[wasm_bindgen_test]
fn in_point_tests() {
    let diagram = base_diagram();
    test_points(diagram, ZERO_POINT);
}

fn reload_data() -> Rc<RefCell<DiagramCore>> {
    let diagram = base_diagram();

    let (node_a, node_b) = nodes_a_b();
    let box_a = box_a();
    let links = vec![default_link_set((1, 2))];

    match diagram
        .borrow_mut()
        .set_data(vec![box_a], vec![node_a, node_b], links)
    {
        Err(_) => panic!("Setting data failed!"),
        Ok(_) => (),
    };
    diagram
}
#[test]
#[wasm_bindgen_test]
fn reload_data_test() {
    reload_data();
}

#[test]
#[wasm_bindgen_test]
fn test_reloaded_points() {
    let diagram = reload_data();
    test_points(diagram, ZERO_POINT);
}

#[test]
#[wasm_bindgen_test]
fn test_move_box() {
    let diagram = reload_data();
    test_points(Rc::clone(&diagram), ZERO_POINT);
    let distance = &Point { x: 5.0, y: 5.0 };

    diagram.borrow_mut().move_nodes(distance, &[0, 1, 2]);
    diagram.borrow_mut().finish_move();

    test_points(diagram, *distance);
}
