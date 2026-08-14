use std::{cell::RefCell, rc::Rc};

use js_sys::Number;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, Event, HtmlCanvasElement, HtmlElement, PointerEvent, WheelEvent,
};

use crate::{
    Point,
    constants::CANVAS_ERROR,
    render::{Render, event_watcher::HtmlEventWatcher},
    square::Square,
};

pub struct Targets {
    pub canvas: HtmlCanvasElement,
    pub context: CanvasRenderingContext2d,

    pub width: u32,
    pub height: u32,
    _on_enter: HtmlEventWatcher,
    _on_move: HtmlEventWatcher,
    _on_down: HtmlEventWatcher,
    _on_up: HtmlEventWatcher,
    _on_leave: HtmlEventWatcher,
    _on_wheel: HtmlEventWatcher,
}

pub fn unpack_canvas(c: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, JsValue> {
    match c.get_context("2d") {
        Ok(o) => match o {
            Some(obj) => match obj.dyn_into::<web_sys::CanvasRenderingContext2d>() {
                Ok(context) => Ok(context),
                Err(_) => Err(JsValue::from(CANVAS_ERROR)),
            },
            None => Err(JsValue::from(CANVAS_ERROR)),
        },
        Err(e) => Err(e),
    }
}

pub fn to_fixed_px(n: f64) -> String {
    let js_num: Number = n.into();
    let js_str = unsafe { js_num.to_fixed(2).unwrap_unchecked() };
    let mut str = String::from(js_str);
    str.push_str("px");
    str
}

fn get_el_xy(e: &Event, div: &HtmlElement) -> Option<Point> {
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

macro_rules! create_mouse_callback {
    ($render:expr,$target:literal,$el:expr,$method:ident) => {{
        let render = $render.clone();
        let div = $el.clone();
        let cb = move |e: Event| {
            if let Some(p) = get_el_xy(&e, &div) {
                render.borrow().$method(&p);
            }
        };
        HtmlEventWatcher::new(&$target, cb, &$el.clone())
    }};
}

impl Targets {
    pub fn new(
        w: u32,
        h: u32,
        render: Rc<RefCell<Render>>,
        canvas: HtmlCanvasElement,
    ) -> Result<Self, JsValue> {
        let rect = canvas.get_bounding_client_rect();
        let screen = Square::new(0.0, 0.0, rect.width(), rect.height());
        let dst = screen.center(&Square::new(0.0, 0.0, w as f64, h as f64));
        canvas.set_width(w);
        canvas.set_height(h);
        let el = canvas.unchecked_ref::<HtmlElement>().clone();
        let on_down = create_mouse_callback!(render, "pointerdown", el, on_mouse_down)?;
        let on_up = create_mouse_callback!(render, "pointerup", el, on_mouse_up)?;
        let on_move = create_mouse_callback!(render, "pointermove", el, on_mouse_move)?;
        let on_leave = create_mouse_callback!(render, "pointerenter", el, on_mouse_leave)?;
        let on_enter = create_mouse_callback!(render, "pointerleave", el, on_mouse_enter)?;

        // -- on_wheel start
        let we = el.clone();
        let on_wheel = HtmlEventWatcher::new(
            "wheel",
            move |e| {
                e.prevent_default();
                e.stop_propagation();
                match (get_el_xy(&e, &we), e.dyn_ref::<WheelEvent>()) {
                    (Some(p), Some(w)) => {
                        render.borrow().on_mouse_wheel(&p, w.delta_y());
                    }
                    _ => (),
                }
            },
            &el,
        )?;
        // -- on_wheel end

        let context = unpack_canvas(&canvas)?;
        return Ok(Self {
            width: w,
            height: h,
            canvas,
            context,
            _on_enter: on_enter,
            _on_move: on_move,
            _on_down: on_down,
            _on_up: on_up,
            _on_leave: on_leave,
            _on_wheel: on_wheel,
        });
    }
}
