use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    mem,
    rc::{Rc, Weak},
};

use crate::{
    ElementOpt, Point, Transform,
    bsp::{IdxBoxAction, ScreenIndex, ScreenSlot},
    constants::*,
    imgcache::ImgCache,
    link::{Bundle, Link, LinkContainer},
    node::Node,
    render::Render,
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
    this: Weak<RefCell<Self>>,
}

pub enum MovedElements {
    Node(u32),
    Box(u32),
    Link(u64),
}

#[wasm_bindgen]
pub struct Diagram {
    core: Rc<RefCell<DiagramCore>>,
}

#[wasm_bindgen]
pub struct DiagramBulkLoad {
    diagram: Rc<RefCell<DiagramCore>>,
    done: bool,
}

#[wasm_bindgen]
impl DiagramBulkLoad {
    pub fn nodes(&self, nodes: Vec<Node>) -> Result<(), JsValue> {
        let mut diagram = self.diagram.borrow_mut();
        diagram.nodes.reserve(nodes.len());
        for node in nodes {
            self._add_node(NodeCanvasTarget::Node(node))?;
        }
        Ok(())
    }
    pub fn boxes(&self, boxes: Vec<Node>) -> Result<(), JsValue> {
        let mut diagram = self.diagram.borrow_mut();
        diagram.nodes.reserve(boxes.len());
        for node in boxes {
            self._add_node(NodeCanvasTarget::Box(node))?;
        }
        Ok(())
    }
    fn _add_node(&self, n: NodeCanvasTarget) -> Result<(), JsValue> {
        self.done_check()?;
        self.diagram.borrow_mut().add_node(n)
    }
    pub fn links(&self, links: Vec<Link>) -> Result<(), JsValue> {
        self.done_check()?;
        let mut diagram = self.diagram.borrow_mut();
        diagram.links.reserve(links.len());
        for link in links {
            diagram.add_link(link)?
        }
        Ok(())
    }

    pub fn eloptions(&self, el_ops: Vec<ElementOpt>) -> Result<(), JsValue> {
        self.done_check()?;
        self.diagram.borrow_mut().set_element_options(el_ops);
        Ok(())
    }
    pub fn bundles(&self, bundles: Vec<Bundle>) -> Result<(), JsValue> {
        self.done_check()?;
        let mut diagram = self.diagram.borrow_mut();
        for bundle in bundles {
            diagram.add_bundle(bundle)?;
        }
        Ok(())
    }
    fn done_check(&self) -> Result<(), JsValue> {
        match self.done {
            true => Err(JsValue::from(BULK_LOAD_ERROR)),
            false => Ok(()),
        }
    }

    pub fn done(&mut self) -> Result<(), JsValue> {
        if self.done {
            return Ok(());
        }
        self.done = true;
        self.diagram.borrow_mut().finish_bulk_load()
    }
}
impl Drop for DiagramBulkLoad {
    fn drop(&mut self) {
        let _ = self.done();
    }
}

impl DiagramBulkLoad {
    pub fn new(diagram: Rc<RefCell<DiagramCore>>) -> Self {
        Self {
            diagram,
            done: false,
        }
    }
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
    pub fn create_loader(&self) -> DiagramBulkLoad {
        self.core.borrow_mut().startbulk_load()
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
        self.el_ops.shrink_to_fit();
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
            this: Weak::new(),
        };

        let this = Rc::new(RefCell::new(res));
        this.borrow_mut().render.borrow_mut().diagram = Rc::downgrade(&this);
        this.borrow_mut().this = Rc::downgrade(&this);

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
        for opt in el_ops {
            cache.load_img(&opt.img);
            self.el_ops.insert(opt.id, opt);
        }
    }

    pub fn add_node(&mut self, n: NodeCanvasTarget) -> Result<(), JsValue> {
        let id;
        {
            let node = n.get();
            self.center.x += node.layout.x;
            self.center.y += node.layout.x;
            id = node.id;
            let ss;
            match &n {
                NodeCanvasTarget::Box(_) => ss = ScreenSlot::Box(id),
                NodeCanvasTarget::Node(_) => ss = ScreenSlot::Node(id),
            }
            self.update_groups(&node.groups, id);
            let points = node.layout.idx(self.render_ops.index_step);
            self.idx.manage(&ss, points, IdxBoxAction::Add);
            self.render_order.push(ss);
        }
        match self.nodes.insert(id, n) {
            Some(_) => Err(JsValue::from(NODE_ADD_ERROR)),
            None => Ok(()),
        }
    }
    pub fn startbulk_load(&mut self) -> DiagramBulkLoad {
        self.clear();
        DiagramBulkLoad::new(unsafe { self.this.upgrade().unwrap_unchecked() })
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

    pub fn move_nodes(&mut self, distance: &Point, node_ids: &[u32]) -> Vec<MovedElements> {
        let mut links = HashSet::new();
        let mut moved = Vec::with_capacity(node_ids.len());

        for node_id in node_ids {
            if let Some(moved) = self.node_links.get(node_id) {
                links.reserve(moved.len());
                for link_id in moved {
                    links.insert(*link_id);
                }
            }

            let node: &mut Node;
            match unsafe { self.nodes.get_mut(node_id).unwrap_unchecked() } {
                NodeCanvasTarget::Box(n) => {
                    moved.push(MovedElements::Box(n.id));
                    node = n;
                }
                NodeCanvasTarget::Node(n) => {
                    moved.push(MovedElements::Node(n.id));
                    node = n;
                }
            }
            node.layout.move_distance(distance);
        }

        moved.reserve(links.len());
        for lid in links {
            let lc = unsafe { self.links.get_mut(&lid).unwrap_unchecked() };
            let (a, b) = lc.get_src_dst();
            let src = unsafe { self.nodes.get(&a).unwrap_unchecked().get() };
            let dst = unsafe { self.nodes.get(&b).unwrap_unchecked().get() };
            lc.update(src, dst, &self.render_ops);
            moved.push(MovedElements::Link(lid));
        }

        moved
    }
}
