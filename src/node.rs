use crate::square::Square;
use wasm_bindgen::prelude::*;
#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
pub struct Node {
    pub layout: Square,
    pub label: String,
    pub opt: u32,
    pub id: u32,
}

#[wasm_bindgen]
impl Node {
    #[wasm_bindgen(constructor)]
    pub fn new(id: u32, layout: Square, label: String, opt: u32) -> Self {
        Self {
            label,
            layout,
            opt,
            id,
        }
    }
}
