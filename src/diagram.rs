use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    mem,
    rc::Rc,
};

use crate::{
    ElementOpt, Point, Transform,
    bsp::{IdxBoxAction, ScreenIndex, ScreenSlot},
    constants::*,
    imgcache::ImgCache,
    link::{Bundle, Link, LinkContainer},
    node::Node,
    render::Render,
    square::Square,
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
    pub index_step: i64,
    pub id: String,
    pub node_font_scale: f64,
}

#[wasm_bindgen(inspectable, getter_with_clone)]
pub struct MovedNodes {
    pub nodes: Vec<MovedNode>,
    pub boxes: Vec<MovedNode>,
}
#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct MovedNode {
    pub id: u32,
    pub layout: Square,
}

#[wasm_bindgen]
impl DiagramOpt {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
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
            index_step: DEFAULT_IDX_STEP,
            id: String::from(DEFAULT_ELEMENT_ID),
            node_font_scale: NODE_FONT_SCALE,
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

#[derive(Hash, PartialEq)]
pub enum NodeCanvasTarget {
    Box(Node),
    Node(Node),
}
impl Eq for NodeCanvasTarget {}
impl NodeCanvasTarget {
    pub fn unwrap(self) -> Node {
        match self {
            NodeCanvasTarget::Box(node) => node,
            NodeCanvasTarget::Node(node) => node,
        }
    }

    pub fn get(&self) -> &Node {
        match self {
            NodeCanvasTarget::Box(node) => node,
            NodeCanvasTarget::Node(node) => node,
        }
    }
    pub fn get_mut(&mut self) -> &mut Node {
        match self {
            NodeCanvasTarget::Box(node) => node,
            NodeCanvasTarget::Node(node) => node,
        }
    }
}
pub struct DiagramCore {
    pub el_ops: HashMap<u32, ElementOpt>,
    pub nodes: HashMap<u32, NodeCanvasTarget>,
    pub links: HashMap<u64, LinkContainer>,
    pub idx: ScreenIndex,
    pub render_order: Vec<ScreenSlot>,
    pub render_ops: DiagramOpt,
    pub center: Point,

    pub transform: Transform,
    pub groups: HashMap<u32, HashSet<u32>>, // group_id,set->node_ids
    pub node_links: HashMap<u32, HashSet<u64>>,
    pub img_cache: Rc<RefCell<ImgCache>>,
    pub render: Rc<RefCell<Render>>,
}

#[wasm_bindgen]
pub struct Diagram {
    core: Rc<RefCell<DiagramCore>>,
}

#[wasm_bindgen]
impl Diagram {
    pub fn new(render_ops: DiagramOpt) -> Self {
        Self {
            core: DiagramCore::new(render_ops),
        }
    }
    pub fn get_transform(&self) -> Transform {
        self.core.borrow().get_transform()
    }
    pub fn set_transform(&self, t: Transform) {
        self.core.borrow_mut().transform = t;
    }

    pub fn set_element_options(&self, el_ops: Vec<ElementOpt>) {
        self.core.borrow_mut().set_element_options(el_ops);
    }
    pub fn set_data(
        &self,
        boxes: Vec<Node>,
        nodes: Vec<Node>,
        links: Vec<Link>,
        bundles: Vec<Bundle>,
    ) -> Result<(), JsValue> {
        self.core
            .borrow_mut()
            .set_data(boxes, nodes, links, bundles)
    }
}
impl DiagramCore {
    pub fn finish_bulk_load(&mut self) -> Result<(), JsValue> {
        let total = self.nodes.len();
        if total == 0 {
            return Ok(());
        }
        let total = total as f64;
        let opt = &self.render_ops;
        let step = opt.index_step;
        self.center.x = self.center.x / total;
        self.center.y = self.center.y / total;
        for (_, lc) in self.links.iter_mut() {
            let (a, b) = lc.get_src_dst();
            let x = unsafe { self.nodes.get(&a).unwrap_unchecked() };
            let y = unsafe { self.nodes.get(&b).unwrap_unchecked() };
            let (src, dst);
            match x {
                NodeCanvasTarget::Box(_) => return Err(JsValue::from_str(LINK_ADD_ERROR)),
                NodeCanvasTarget::Node(node) => src = node,
            }
            match y {
                NodeCanvasTarget::Box(_) => return Err(JsValue::from_str(LINK_ADD_ERROR)),
                NodeCanvasTarget::Node(node) => dst = node,
            }
            lc.update(src, dst, opt);
            let dd = unsafe { lc.draw_data.as_ref().unwrap_unchecked() };
            let points = dd.index.idx(step);
            self.idx
                .manage(&ScreenSlot::Link(lc.id), points, IdxBoxAction::Add);
        }
        self.links.shrink_to_fit();
        self.nodes.shrink_to_fit();
        self.node_links.shrink_to_fit();
        self.groups.shrink_to_fit();
        Ok(())
    }
    pub fn add_link(&mut self, link: Link) -> Result<(), JsValue> {
        if link.src == link.dst
            || !self.nodes.contains_key(&link.src) && !self.nodes.contains_key(&link.dst)
        {
            return Err(JsValue::from("Cannot add Link to node that does not exist"));
        }
        self.get_lc(link.link_id()).add_link(link);
        Ok(())
    }

    pub fn add_bundle(&mut self, bundle: Bundle) -> Result<(), JsValue> {
        if bundle.src == bundle.dst
            || !self.nodes.contains_key(&bundle.src) && !self.nodes.contains_key(&bundle.dst)
        {
            return Err(JsValue::from(
                "Cannot add Bundle to node that does not exist",
            ));
        }
        self.get_lc(bundle.link_id()).add_bundle(bundle)?;
        Ok(())
    }
    pub fn new(render_ops: DiagramOpt) -> Rc<RefCell<Self>> {
        let render = Rc::new(RefCell::new(Render::new()));
        let res = Self {
            nodes: HashMap::new(),
            links: HashMap::new(),
            el_ops: HashMap::new(),
            idx: ScreenIndex::new(render_ops.index_step),
            render_order: Vec::new(),
            render_ops,
            center: ZERO_POINT,
            transform: ZERO_TRANSFORM,
            groups: HashMap::new(),
            node_links: HashMap::new(),
            img_cache: ImgCache::new(Rc::clone(&render)),
            render,
        };

        let this = Rc::new(RefCell::new(res));
        this.borrow_mut().render.borrow_mut().diagram = Rc::downgrade(&this);

        this
    }

    pub fn set_transform(&mut self, t: Transform) {
        self.transform = t;
    }
    pub fn get_transform(&self) -> Transform {
        self.transform
    }
    pub fn set_element_options(&mut self, el_ops: Vec<ElementOpt>) {
        let mut cache = self.img_cache.borrow_mut();
        self.el_ops.reserve(el_ops.len());
        cache.cache.reserve(el_ops.len());
        for opt in el_ops {
            cache.load_img(&opt.img);
            self.el_ops.insert(opt.id, opt);
        }

        cache.cache.shrink_to_fit();
        self.el_ops.shrink_to_fit();
    }

    pub fn add_node(&mut self, node: Node, as_node: bool) -> Result<(), JsValue> {
        let id = node.id;
        self.center.x += node.layout.x;
        self.center.y += node.layout.x;

        self.update_groups(&node.groups, id);
        let (n, ss);
        let points = node.layout.idx(self.render_ops.index_step);
        match as_node {
            true => (n, ss) = (NodeCanvasTarget::Node(node), ScreenSlot::Node(id)),
            false => (n, ss) = (NodeCanvasTarget::Box(node), ScreenSlot::Box(id)),
        }
        self.idx.manage(&ss, points, IdxBoxAction::Add);
        self.render_order.push(ss);
        match self.nodes.insert(id, n) {
            Some(_) => Err(JsValue::from(NODE_ADD_ERROR)),
            None => Ok(()),
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
        for node in boxes {
            self.add_node(node, false)?;
        }
        for node in nodes {
            self.add_node(node, true)?;
        }
        for link in links {
            self.add_link(link)?
        }
        for bundle in bundles {
            self.add_bundle(bundle)?;
        }
        self.finish_bulk_load()?;

        Ok(())
    }
    fn get_lc<'l>(&'l mut self, id: u64) -> &'l mut LinkContainer {
        if let Some(c) = self.links.get_mut(&id) {
            return unsafe { mem::transmute(c) };
        }

        self.links.insert(id, LinkContainer::new(id));
        let res = unsafe { self.links.get_mut(&id).unwrap_unchecked() };
        self.render_order.push(ScreenSlot::Link(id));

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
        self.center = ZERO_POINT;
    }

    pub fn get_related_nodes(&self, node_ids: &[u32]) -> Vec<u32> {
        let mut ids = HashSet::with_capacity(node_ids.len());
        for node_id in node_ids {
            ids.insert(*node_id);
            let node = unsafe { self.nodes.get(node_id).unwrap_unchecked().get() };
            for gid in &node.groups {
                let group = unsafe { self.groups.get(gid).unwrap_unchecked() };
                for node_id in group {
                    ids.insert(*node_id);
                }
            }
        }
        ids.into_iter().collect()
    }

    pub fn move_nodes(&mut self, distance: &Point, node_ids: &[u32]) -> MovedNodes {
        let mut nodes = Vec::new();
        let mut boxes = Vec::new();
        let mut links = HashSet::new();
        let step = self.idx.step;

        for node_id in node_ids {
            if let Some(moved) = self.node_links.get(node_id) {
                links.reserve(moved.len());
                for link_id in moved {
                    links.insert(*link_id);
                }
            }

            let t = unsafe { self.nodes.get_mut(node_id).unwrap_unchecked() };
            let (old, new);
            {
                let node = t.get_mut();
                old = node.layout.idx(step);
                node.layout.move_distance(distance);
                new = node.layout.idx(step);
            }
            match t {
                NodeCanvasTarget::Box(n) => {
                    boxes.push(MovedNode {
                        id: n.id,
                        layout: n.layout,
                    });
                    self.idx.update(&ScreenSlot::Box(*node_id), old, new);
                }
                NodeCanvasTarget::Node(n) => {
                    nodes.push(MovedNode {
                        id: n.id,
                        layout: n.layout,
                    });
                    self.idx.update(&ScreenSlot::Node(*node_id), old, new);
                }
            }
        }

        for lid in links {
            let lc = unsafe { self.links.get_mut(&lid).unwrap_unchecked() };
            let dd = unsafe { lc.draw_data.as_mut().unwrap_unchecked() };
            let old = dd.index.idx(step);
            dd.move_distance(distance);
            let new = dd.index.idx(step);
            self.idx.update(&ScreenSlot::Link(lid), old, new);
        }
        MovedNodes { nodes, boxes }
    }
}
