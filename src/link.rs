use std::mem;

use wasm_bindgen::prelude::*;

use crate::{Point, square::Square, utils::create_container_id};

#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Animation {
    Both,  // Animate in both directions
    ToSrc, // Animate towards the src node
    ToDst, // Animate towards the dst node
    None,  // Do not animate
}
#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
pub struct Link {
    pub src: u32,
    pub dst: u32,
    pub opt: u32,
    pub label: String,
    pub animation: Animation,
}
#[wasm_bindgen]
impl Link {
    #[wasm_bindgen(constructor)]
    pub fn new(src: u32, dst: u32, opt: u32, label: String, animation: Animation) -> Self {
        Self {
            src,
            dst,
            opt,
            label,
            animation,
        }
    }
    pub fn link_id(&self) -> u64 {
        create_container_id(self.src, self.dst)
    }
}
#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
pub struct Bundle {
    pub src: u32,
    pub dst: u32,
    pub opt: u32,
    pub label: String,
    pub links: Vec<usize>,
}
#[wasm_bindgen]
impl Bundle {
    #[wasm_bindgen(constructor)]
    pub fn new(src: u32, dst: u32, opt: u32, label: String, links: Vec<usize>) -> Self {
        Self {
            src,
            dst,
            opt,
            label,
            links,
        }
    }
    pub fn link_id(&self) -> u64 {
        create_container_id(self.src, self.dst)
    }
}

pub struct DrawData {
    pub line_width: f64,

    pub bundle_size: (f64, f64), // (width,height)
    pub bunldes: Vec<(Point, u32)>,
    pub links: Vec<(Point, Point, u32)>,
    pub animations: Vec<(Point, Point)>,
    pub index: Square,
}
pub struct LinkContainer {
    pub links: Vec<Link>,
    pub bundles: Vec<Bundle>,
    pub draw_data: Option<DrawData>,
    pub id: u64,
}

impl LinkContainer {
    pub fn new(id: u64) -> Self {
        Self {
            links: Vec::new(),
            bundles: Vec::new(),
            draw_data: None,
            id,
        }
    }
    pub fn get_src_dst(&self) -> (u32, u32) {
        return unsafe { mem::transmute::<u64, (u32, u32)>(self.id) };
    }
    pub fn add_link(&mut self, link: Link) {
        self.links.push(link);
    }

    pub fn add_bundle(&mut self, bundle: Bundle) -> Result<(), JsValue> {
        for id in &bundle.links {
            if !(self.links.len() > *id) {
                return Err(JsValue::from_str("Bundle Points to an invalid link"));
            }
        }
        self.bundles.push(bundle);
        Ok(())
    }
}
