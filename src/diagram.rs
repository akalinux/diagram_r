use std::{collections::HashMap, mem};

use crate::{
    ElementOpt, Point, Transform,
    bsp::{IdxBoxAction, ScreenIndex, ScreenSlot},
    constants::*,
    link::{Bundle, Link, LinkContainer},
    node::Node,
};
use js_sys::Function;
use wasm_bindgen::prelude::*;
#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
pub struct DiagramOpt {
    pub wheel_move: f64,
    pub highlight_scale: f64,
    pub timeout: i32,
    pub font_family: String,
    pub text_align: String,
    pub animation_dashes: Vec<f64>,
    pub highlight_alpha: f64,
    pub highlight_color: String,
    pub bulk_img_update: bool,
    pub callback: Option<Function>,
}

#[wasm_bindgen]
impl DiagramOpt {
    #[wasm_bindgen(constructor)]
    pub fn new(
        wheel_move: f64,
        highlight_scale: f64,
        timeout: i32,
        font_family: String,
        text_align: String,
        animation_dashes: Vec<f64>,
        highlight_alpha: f64,
        highlight_color: String,
        bulk_img_update: bool,
        callback: Option<Function>,
    ) -> Self {
        Self {
            wheel_move,
            highlight_scale,
            timeout,
            font_family,
            text_align,
            highlight_alpha,
            highlight_color,
            bulk_img_update,
            callback,
            animation_dashes,
        }
    }
    pub fn defaults() -> Self {
        Self {
            wheel_move: DEFAULT_SCREEN_ZOOM,
            highlight_scale: DEFAULT_HIGHLIGHT_SCALE,
            timeout: DEFAULT_HOVER_TIMEOUT,
            font_family: String::from(DEFAULT_FONT_FAMILY),
            text_align: String::from(DEFAULT_TEXT_ALIGN),
            animation_dashes: Vec::from(DEFAULT_ANIMATION_DASHES),
            highlight_alpha: DEFAULT_HIGHLIGHT_ALPHA,
            highlight_color: String::from(DEFAULT_HIGHLIGHT_COLOR),
            bulk_img_update: true,
            callback: None,
        }
    }
}

pub enum NodeLayer {
    Container(Node),
    Map(Node),
}
#[wasm_bindgen]
pub struct Diagram {
    #[wasm_bindgen(skip)]
    pub el_ops: HashMap<u32, ElementOpt>,
    #[wasm_bindgen(skip)]
    pub nodes: HashMap<u32, NodeLayer>,
    #[wasm_bindgen(skip)]
    pub links: HashMap<u64, LinkContainer>,
    #[wasm_bindgen(skip)]
    pub idx: ScreenIndex,

    #[wasm_bindgen(skip)]
    pub render_order: Vec<u32>,
    #[wasm_bindgen(skip)]
    pub render_ops: DiagramOpt,
    #[wasm_bindgen(skip)]
    pub center: Point,
    #[wasm_bindgen(skip)]
    pub transform: Transform,
}
#[wasm_bindgen]
impl Diagram {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            links: HashMap::new(),
            el_ops: HashMap::new(),
            idx: ScreenIndex::new(DEFAULT_IDX_STEP),
            render_order: Vec::new(),
            render_ops: DiagramOpt::defaults(),
            center: ZERO_POINT,
            transform: ZERO_TRANSFORM,
        }
    }

    pub fn set_render_options(&mut self, ops: DiagramOpt) {
        self.render_ops = ops;
    }
    pub fn get_render_options(&mut self) -> DiagramOpt {
        self.render_ops.clone()
    }
    pub fn set_transform(&mut self, t: Transform) {
        self.transform = t;
    }
    pub fn get_transform(&self) -> Transform {
        self.transform
    }
    pub fn set_data(
        &mut self,
        boxes: Vec<Node>,
        nodes: Vec<Node>,
        links: Vec<Link>,
        bundles: Vec<Bundle>,
    ) -> Result<(), JsValue> {
        self.clear();

        let step = self.idx.step;
        let mut x = 0.0;
        let mut y = 0.0;
        for n in nodes {
            let points = n.layout.idx(step);
            x += n.layout.x;
            y += n.layout.y;
            self.render_order.push(n.id);
            self.idx
                .manage(&ScreenSlot::Node(n.id), points, IdxBoxAction::Add);
            match self.nodes.insert(n.id, NodeLayer::Container(n)) {
                Some(_) => return Err(JsValue::from_str(NODE_ADD_ERROR)),
                None => (),
            }
        }
        if self.nodes.len() != 0 {
            x = x / self.nodes.len() as f64;
            y = y / self.nodes.len() as f64;
        }
        self.center = Point { x, y };
        for b in boxes {
            self.render_order.push(b.id);
            let points = b.layout.idx(step);
            self.idx
                .manage(&ScreenSlot::Box(b.id), points, IdxBoxAction::Add);

            match self.nodes.insert(b.id, NodeLayer::Container(b)) {
                Some(_) => return Err(JsValue::from_str(NODE_ADD_ERROR)),
                None => (),
            }
        }
        for link in links {
            if link.src == link.dst
                || !self.nodes.contains_key(&link.src) && !self.nodes.contains_key(&link.dst)
            {
                return Err(JsValue::from("Cannot add Link to node that does not exist"));
            }
            self.get_lc(link.link_id()).add_link(link);
        }
        for bundle in bundles {
            if bundle.src == bundle.dst
                || !self.nodes.contains_key(&bundle.src) && !self.nodes.contains_key(&bundle.dst)
            {
                return Err(JsValue::from(
                    "Cannot add Bundle to node that does not exist",
                ));
            }
            self.get_lc(bundle.link_id()).add_bundle(bundle)?;
        }

        for lc in self.links.iter_mut() {}
        Ok(())
    }
}

impl Diagram {
    fn get_lc<'l>(&'l mut self, id: u64) -> &'l mut LinkContainer {
        if let Some(c) = self.links.get_mut(&id) {
            return unsafe { mem::transmute(c) };
        }
        self.links.insert(id, LinkContainer::new(id));
        unsafe { self.links.get_mut(&id).unwrap_unchecked() }
    }
    fn clear(&mut self) {
        self.nodes.clear();
        self.links.clear();
        self.el_ops.clear();
        self.idx.clear();
        self.render_order.clear();
    }
}
