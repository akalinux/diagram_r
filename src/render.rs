use std::{cell::RefCell, rc::Weak};
pub mod canvasrender;
pub mod event_watcher;
pub mod size_watcher;
pub mod timeout;
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;
pub mod pointerwatcher;
use crate::{Point, bsp::ScreenSlot, diagram::DiagramCore};

pub trait BuildRender {
    fn new(
        canvas: HtmlCanvasElement,
        diagram: Weak<RefCell<DiagramCore>>,
    ) -> Result<Box<dyn CoreRender>, JsValue>;
}

pub trait CoreRender {
    fn render(&self) -> Result<(), JsValue>;
    fn update(&self, target: ScreenSlot, distance: &Point);
    fn clear(&self);
    fn get_width_height(&self) -> (f32, f32);
}
