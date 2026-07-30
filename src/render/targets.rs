use std::{cell::RefCell, rc::Rc};

use js_sys::Number;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, Document, Event, HtmlCanvasElement, HtmlElement, PointerEvent,
    ResizeObserver, ResizeObserverEntry, WheelEvent, Window,
};

use crate::{
    Point,
    constants::{CANVAS_ERROR, DEFAULT_CANVAS_STYLE, DOM_ERROR, EL_ERROR, WINDOW_ERROR},
    render::{Render, event_watcher::HtmlEventWatcher, size_watcher::SizeWatcher},
    square::Square,
};

pub struct Targets {
    pub boxes: CanvasRenderingContext2d,
    pub links: CanvasRenderingContext2d,
    pub animations: CanvasRenderingContext2d,
    pub nodes: CanvasRenderingContext2d,
    pub highlight: CanvasRenderingContext2d,
    pub window: Window,
    pub width: u32,
    pub height: u32,
    _on_enter: HtmlEventWatcher,
    _on_move: HtmlEventWatcher,
    _on_down: HtmlEventWatcher,
    _on_up: HtmlEventWatcher,
    _on_leave: HtmlEventWatcher,
    _on_wheel: HtmlEventWatcher,
    _on_size: SizeWatcher,
}

pub fn create_canvas(
    dom: &Document,
    div: &HtmlElement,
    w: u32,
    h: u32,
    top: &String,
    left: &String,
) -> Result<HtmlCanvasElement, JsValue> {
    let c = dom
        .create_element("canvas")?
        .dyn_into::<HtmlCanvasElement>()?;
    c.set_width(w);
    c.set_height(h);
    let style = c.style();
    c.set_attribute("style", DEFAULT_CANVAS_STYLE)?;
    style.set_property("width", &to_fixed_px(w as f64))?;
    style.set_property("height", &to_fixed_px(h as f64))?;
    style.set_property("top", top)?;
    style.set_property("left", left)?;
    div.append_child(&c)?;

    Ok(c)
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
    pub fn new(w: u32, h: u32, render: Rc<RefCell<Render>>, id: String) -> Result<Self, JsValue> {
        let (window, dom, div);
        match web_sys::window() {
            Some(w) => match w.document() {
                None => return Err(JsValue::from(DOM_ERROR)),
                Some(d) => match d.get_element_by_id(&id) {
                    Some(e) => match e.dyn_into::<HtmlElement>() {
                        Ok(el) => (window, dom, div) = (w, d, el),
                        Err(_) => return Err(JsValue::from(EL_ERROR)),
                    },
                    None => return Err(JsValue::from(EL_ERROR)),
                },
            },
            None => return Err(JsValue::from(WINDOW_ERROR)),
        };
        let rect = div.get_bounding_client_rect();
        let screen = Square::new(0.0, 0.0, rect.width(), rect.height());
        let dst = screen.center(&Square::new(0.0, 0.0, w as f64, h as f64));
        let top = to_fixed_px(dst.y);
        let left = to_fixed_px(dst.x);
        let hboxes = create_canvas(&dom, &div, w, h, &top, &left)?;
        let hlinks = create_canvas(&dom, &div, w, h, &top, &left)?;
        let hanimations = create_canvas(&dom, &div, w, h, &top, &left)?;
        let hnodes = create_canvas(&dom, &div, w, h, &top, &left)?;
        let hhighlight = create_canvas(&dom, &div, w, h, &top, &left)?;

        let on_down = create_mouse_callback!(render, "pointerdown", div, on_mouse_down)?;
        let on_up = create_mouse_callback!(render, "pointerup", div, on_mouse_up)?;
        let on_move = create_mouse_callback!(render, "pointermove", div, on_mouse_move)?;
        let on_leave = create_mouse_callback!(render, "pointerenter", div, on_mouse_leave)?;
        let on_enter = create_mouse_callback!(render, "pointerleave", div, on_mouse_enter)?;

        // -- on_wheel start
        let on_wheel = HtmlEventWatcher::new(
            "wheel",
            move |e| {
                e.prevent_default();
                e.stop_propagation();
                if let Some(w) = e.dyn_ref::<WheelEvent>() {
                    render.borrow().on_mouse_wheel(w.delta_y());
                }
            },
            &div,
        )?;
        // -- on_wheel end

        let boxes = unpack_canvas(&hboxes)?;
        let links = unpack_canvas(&hlinks)?;
        let animations = unpack_canvas(&hanimations)?;
        let nodes = unpack_canvas(&hnodes)?;
        let highlight = unpack_canvas(&hhighlight)?;

        let mut last_width = rect.width();
        let mut last_height = rect.height();
        let d = div.clone();
        // elements move ownership here into the callback
        let cb = move |_entries: Vec<ResizeObserverEntry>, _observer: ResizeObserver| {
            let rect = d.get_bounding_client_rect();
            if last_width == rect.width() && last_height == rect.height() {
                return;
            }
            last_height = rect.height();
            last_width = rect.width();
            let container = Square::new(0.0, 0.0, last_width, last_height);
            let center = container.center(&Square::new(0.0, 0.0, w as f64, h as f64));
            let top = to_fixed_px(center.y);
            let left = to_fixed_px(center.x);

            for c in [&hboxes, &hlinks, &hanimations, &hnodes, &hhighlight] {
                let style = c.style();
                let _ = style.set_property("top", &top);
                let _ = style.set_property("left", &left);
            }
        };

        let on_size = SizeWatcher::new(&div, cb)?;

        return Ok(Self {
            width: w,
            height: h,
            boxes,
            links,
            animations,
            nodes,
            highlight,
            window,
            _on_enter: on_enter,
            _on_move: on_move,
            _on_down: on_down,
            _on_up: on_up,
            _on_leave: on_leave,
            _on_wheel: on_wheel,
            _on_size: on_size,
        });
    }
}
