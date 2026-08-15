use std::{cell::RefCell, rc::Weak};

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Event, HtmlCanvasElement, PointerEvent, WheelEvent};

use crate::{
    Point,
    diagram::{self, DiagramCore},
    render::event_watcher::HtmlEventWatcher,
};
macro_rules! create_mouse_callback {
    ($render:expr,$target:literal,$el:expr,$method:ident) => {{
        let render = unsafe { $render.upgrade().unwrap_unchecked() }.clone();
        let div = $el.clone();
        let cb = move |e: Event| {
            if let Some(p) = get_el_xy(&e, &div) {
                render.borrow().$method(&p);
            }
        };
        HtmlEventWatcher::new(&$target, cb, &$el.clone())
    }};
}

fn get_el_xy(e: &Event, div: &HtmlCanvasElement) -> Option<Point> {
    e.prevent_default();
    e.stop_propagation();
    let rect = div.get_bounding_client_rect();
    if let Some(e) = e.dyn_ref::<PointerEvent>() {
        let x = e.client_x() as f64 - rect.left();
        let y = e.client_y() as f64 - rect.top();
        return Some(Point { x, y });
    }
    None
}

pub struct PointerWatcher {
    _on_enter: HtmlEventWatcher,
    _on_move: HtmlEventWatcher,
    _on_down: HtmlEventWatcher,
    _on_up: HtmlEventWatcher,
    _on_leave: HtmlEventWatcher,
    _on_wheel: HtmlEventWatcher,
}

impl PointerWatcher {
    pub fn new(
        diagram: Weak<RefCell<DiagramCore>>,
        el: HtmlCanvasElement,
    ) -> Result<Self, JsValue> {
        let on_down = create_mouse_callback!(diagram, "pointerdown", el, on_mouse_down)?;
        let on_up = create_mouse_callback!(diagram, "pointerup", el, on_mouse_up)?;
        let on_move = create_mouse_callback!(diagram, "pointermove", el, on_mouse_move)?;
        let on_leave = create_mouse_callback!(diagram, "pointerenter", el, on_mouse_leave)?;
        let on_enter = create_mouse_callback!(diagram, "pointerleave", el, on_mouse_enter)?;
        // -- on_wheel start
        let we = el.clone();
        let diagram = unsafe { diagram.upgrade().unwrap_unchecked() };
        let on_wheel = HtmlEventWatcher::new(
            "wheel",
            move |e| {
                e.prevent_default();
                e.stop_propagation();
                match e.dyn_ref::<WheelEvent>() {
                    Some(w) => {
                        diagram.borrow().on_mouse_wheel(w.delta_y());
                    }
                    _ => (),
                }
            },
            &el,
        )?;
        // -- on_wheel end
        Ok(Self {
            _on_enter: on_enter,
            _on_move: on_move,
            _on_down: on_down,
            _on_up: on_up,
            _on_leave: on_leave,
            _on_wheel: on_wheel,
        })
    }
}
