use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use wasm_bindgen::JsValue;

use crate::{diagram::DiagramCore, imgcache::ImgCache};

pub struct Render {
    pub diagram: Weak<RefCell<DiagramCore>>,
}

impl Render {
    pub fn new() -> Self {
        Self {
            diagram: Weak::new(),
        }
    }
    pub fn on_img(&mut self, _cache: &ImgCache) {}
    pub fn render(&mut self) -> Result<(), JsValue> {
        Ok(())
    }
    pub fn diagram(&self) -> Rc<RefCell<DiagramCore>> {
        unsafe { self.diagram.upgrade().unwrap_unchecked() }
    }
}
