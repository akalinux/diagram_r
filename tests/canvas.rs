#![cfg(test)]
use std::{cell::RefCell, rc::Rc};

use crate::common::{box_a, default_link_set, nodes_a_b};
use diagram_r::{DiagramOpt, diagram::DiagramCore};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
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

wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen_test]
fn test_canvas_2d_context() {
    let window = web_sys::window().expect("should have a window in this environment");
    let document = window.document().expect("should have a document on window");

    // Create a new canvas element dynamically
    let canvas_element = document.create_element("canvas").unwrap();

    let canvas: web_sys::HtmlCanvasElement = canvas_element
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("element should be an HtmlCanvasElement");

    canvas.set_id("test_id");
    canvas.set_width(800);
    canvas.set_height(600);
    let body = unsafe { document.body().unwrap_unchecked() };
    body.append_child(&canvas).unwrap();
    let diagram = base_diagram();
    match diagram.borrow().mount(String::from("test_id")) {
        Ok(_) => (),
        Err(err) => panic!("{}", &unsafe { err.as_string().unwrap_unchecked() }),
    }
}
