use std::{
    collections::{HashMap, HashSet},
    mem,
};

use crate::{
    ElementOpt, Point, Transform,
    bsp::{IdxBoxAction, ScreenIndex, ScreenSlot},
    constants::*,
    link::{Bundle, Link, LinkContainer},
    node::Node,
};
use js_sys::{Array, Function};
use wasm_bindgen::prelude::*;
#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
pub struct DiagramOpt {
    pub wheel_move: f64,
    pub timeout: i32,
    pub font_family: String,
    pub text_align: String,
    pub animation_dashes: Vec<f64>,
    pub highlight_alpha: f64,
    pub highlight_color: String,
    pub highlight_scale: f64,
    pub bulk_img_update: bool,
    pub link_scale: f64,
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
        link_scale: f64,
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
            link_scale,
        }
    }
    pub fn defaults() -> Self {
        Self {
            wheel_move: DEFAULT_SCREEN_ZOOM,
            timeout: DEFAULT_HOVER_TIMEOUT,
            font_family: String::from(DEFAULT_FONT_FAMILY),
            text_align: String::from(DEFAULT_TEXT_ALIGN),
            animation_dashes: Vec::from(DEFAULT_ANIMATION_DASHES),
            highlight_alpha: DEFAULT_HIGHLIGHT_ALPHA,
            highlight_color: String::from(DEFAULT_HIGHLIGHT_COLOR),
            highlight_scale: DEFAULT_HIGHLIGHT_SCALE,
            bulk_img_update: true,
            callback: None,
            link_scale: DEFAULT_LINK_SCALE,
        }
    }
}

impl DiagramOpt {
    pub fn animation_dash(&self) -> Array {
        let res = Array::new();
        for dash in &self.animation_dashes {
            res.push(&JsValue::from_f64(*dash));
        }
        res
    }
}
pub enum NodeLayer {
    Box(Node),
    Node(Node),
}
impl NodeLayer {
    pub fn unwrap(&self) -> &Node {
        match self {
            NodeLayer::Box(node) => node,
            NodeLayer::Node(node) => node,
        }
    }
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
    #[wasm_bindgen(skip)]
    pub groups: HashMap<u32, HashSet<u32>>, // group_id,set->node_ids

    #[wasm_bindgen(skip)]
    pub node_links: HashMap<u32, HashSet<u64>>,
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
            groups: HashMap::new(),
            node_links: HashMap::new(),
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
    pub fn set_render_opts(&mut self, el_ops: Vec<ElementOpt>) {
        for opt in el_ops {
            self.el_ops.insert(opt.id, opt);
        }
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
        self.nodes.reserve(nodes.len() + boxes.len());
        for n in nodes {
            self.render_order.push(n.id);
            self.update_groups(&n.groups, n.id);
            let points = n.layout.idx(step);
            x += n.layout.x;
            y += n.layout.y;
            self.idx
                .manage(&ScreenSlot::Node(n.id), points, IdxBoxAction::Add);
            match self.nodes.insert(n.id, NodeLayer::Node(n)) {
                Some(_) => return Err(JsValue::from_str(NODE_ADD_ERROR)),
                None => (),
            }
        }
        for b in boxes {
            x += b.layout.x;
            y += b.layout.y;
            self.render_order.push(b.id);
            self.update_groups(&b.groups, b.id);
            let points = b.layout.idx(step);
            self.idx
                .manage(&ScreenSlot::Box(b.id), points, IdxBoxAction::Add);

            match self.nodes.insert(b.id, NodeLayer::Box(b)) {
                Some(_) => return Err(JsValue::from_str(NODE_ADD_ERROR)),
                None => (),
            }
        }
        if self.nodes.len() != 0 {
            x = x / self.nodes.len() as f64;
            y = y / self.nodes.len() as f64;
        }
        self.center = Point { x, y };
        self.links.reserve(links.len());
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

        let opt = &self.render_ops;
        for (_, lc) in self.links.iter_mut() {
            let (a, b) = lc.get_src_dst();
            let src = unsafe { self.nodes.get(&a).unwrap_unchecked().unwrap() };
            let dst = unsafe { self.nodes.get(&b).unwrap_unchecked().unwrap() };
            lc.update(src, dst, opt);
            let dd = unsafe { lc.draw_data.as_ref().unwrap_unchecked() };
            let points = dd.index.idx(step);
            self.idx
                .manage(&ScreenSlot::Link(lc.id), points, IdxBoxAction::Add);
        }
        Ok(())
    }
}

impl Diagram {
    fn get_lc<'l>(&'l mut self, id: u64) -> &'l mut LinkContainer {
        if let Some(c) = self.links.get_mut(&id) {
            return unsafe { mem::transmute(c) };
        }

        self.links.insert(id, LinkContainer::new(id));
        let res = unsafe { self.links.get_mut(&id).unwrap_unchecked() };

        let (src, dst) = res.get_src_dst();
        for id in [src, dst] {
            if let Some(l) = self.node_links.get_mut(&id) {
                l.insert(res.id);
            } else {
                let l = HashSet::from([res.id]);
                self.node_links.insert(id, l);
            }
        }

        res
    }

    fn update_groups(&mut self, groups: &Vec<u32>, id: u32) {
        for group in groups {
            let set;
            if let Some(s) = self.groups.get_mut(group) {
                set = s;
            } else {
                let s = HashSet::new();
                self.groups.insert(*group, s);
                set = unsafe { self.groups.get_mut(group).unwrap_unchecked() }
            }
            set.insert(id);
        }
    }
    fn clear(&mut self) {
        self.nodes.clear();
        self.links.clear();
        self.el_ops.clear();
        self.idx.clear();
        self.render_order.clear();
        self.groups.clear();
        self.node_links.clear();
    }

    pub fn get_related_nodes(&self, node_id: u32) -> Vec<u32> {
        let mut ids = HashSet::from([node_id]);
        let node = unsafe { self.nodes.get(&node_id).unwrap_unchecked().unwrap() };
        for gid in &node.groups {
            let group = unsafe { self.groups.get(gid).unwrap_unchecked() };
            for node_id in group {
                ids.insert(*node_id);
            }
        }
        ids.into_iter().collect()
    }
}
