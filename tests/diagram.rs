#![cfg(test)]

use std::{cell::RefCell, rc::Rc};

use diagram_r::{
    DiagramOpt, Point,
    bsp::LookupPointResult,
    constants::ZERO_POINT,
    diagram::{DiagramCore, GroupID, HighlightTargets, LinkAndElement},
};
use wasm_bindgen_test::wasm_bindgen_test;

use crate::common::{box_a, default_link_set, nodes_a_b};
mod common;

pub fn base_diagram() -> Rc<RefCell<DiagramCore>> {
    let mut ops = DiagramOpt::new();
    ops.index_step = 1;
    let ct = DiagramCore::new(ops);

    let (node_a, node_b) = nodes_a_b();
    let links = vec![default_link_set((0, 1))];

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
fn set_data_test() {
    base_diagram();
}
#[test]
#[wasm_bindgen_test]
fn test_highlights() {
    let diagram = base_diagram();
    let d = &*diagram.borrow();

    assert_eq!(
        d.get_highlights(&d.contains_point(&ZERO_POINT)),
        HighlightTargets {
            nodes: vec![0],
            links: vec![],
            boxes: vec![],
            bundles: vec![]
        }
    );
    assert_eq!(
        d.get_highlights(&d.contains_point(&Point { x: 10.0, y: 0.000 })),
        HighlightTargets {
            nodes: vec![1],
            links: vec![],
            boxes: vec![],
            bundles: vec![]
        }
    );
    assert_eq!(
        d.get_highlights(&d.contains_point(&Point { x: 5.0, y: 0.0 })),
        HighlightTargets {
            nodes: vec![],
            links: vec![],
            boxes: vec![0],
            bundles: vec![]
        }
    );
    assert_eq!(
        d.get_highlights(&d.contains_point(&Point { x: 2.5, y: 0.21 })),
        HighlightTargets {
            nodes: vec![0, 1],
            links: vec![LinkAndElement {
                link: 0,
                element: 0
            }],
            boxes: vec![],
            bundles: vec![]
        }
    );
    assert_eq!(
        d.get_highlights(&d.contains_point(&Point { x: 2.5, y: 0.66 })),
        HighlightTargets {
            nodes: vec![0, 1],
            links: vec![LinkAndElement {
                link: 0,
                element: 1
            }],
            boxes: vec![],
            bundles: vec![]
        }
    );
    assert_eq!(
        d.get_highlights(&d.contains_point(&Point { x: 5.0, y: 0.5 })),
        HighlightTargets {
            nodes: vec![0, 1],
            links: vec![
                LinkAndElement {
                    link: 0,
                    element: 0
                },
                LinkAndElement {
                    link: 0,
                    element: 1
                },
            ],
            boxes: vec![],
            bundles: vec![LinkAndElement {
                link: 0,
                element: 0
            },]
        }
    );
}
fn test_points(diagram: Rc<RefCell<DiagramCore>>, p: Point) {
    assert_eq!(
        diagram
            .borrow()
            .contains_point(&(ZERO_POINT.add_distance(&p))),
        LookupPointResult::Node(0)
    );

    assert_eq!(
        diagram
            .borrow()
            .contains_point(&Point { x: 10.0, y: 0.000 }.add_distance(&p),),
        LookupPointResult::Node(1)
    );
    assert_eq!(
        diagram
            .borrow()
            .contains_point(&Point { x: 5.0, y: 0.0 }.add_distance(&p),),
        LookupPointResult::Box(0)
    );
    assert_eq!(
        diagram
            .borrow()
            .contains_point(&Point { x: 5.0, y: 0.5 }.add_distance(&p),),
        LookupPointResult::Bundle((0, 0))
    );
    assert_eq!(
        diagram
            .borrow()
            .contains_point(&Point { x: 2.5, y: 0.21 }.add_distance(&p),),
        LookupPointResult::Link((0, 0))
    );

    assert_eq!(
        diagram
            .borrow()
            .contains_point(&Point { x: 2.5, y: 0.66 }.add_distance(&p),),
        LookupPointResult::Link((0, 1))
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
    let links = vec![default_link_set((0, 1))];

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

    diagram.borrow_mut().move_nodes(
        distance,
        &[GroupID::Box(0), GroupID::Node(0), GroupID::Node(1)],
    );
    diagram.borrow_mut().finish_move();

    test_points(diagram, *distance);
}
