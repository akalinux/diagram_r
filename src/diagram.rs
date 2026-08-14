use rustc_hash::{FxHashMap, FxHashSet};
use std::{cell::RefCell, rc::Rc};
use web_sys::HtmlCanvasElement;

use crate::{
    ElementOpt, Point, Transform,
    bsp::{IndexXY, LookupPointResult, ScreenIndex, ScreenSlot, iter::IdxBoxAction},
    constants::*,
    imgcache::ImgCache,
    link::{LinkContainer, LinkSet},
    node::Node,
    render::{HighlightTargets, Render, UiEvent},
    square::Square,
};
use js_sys::{Array, Function};
use wasm_bindgen::{convert::TryFromJsValue, prelude::*};
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
    pub node_font_scale: f64,
    pub animation_color: String,
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
            node_font_scale: NODE_FONT_SCALE,
            animation_color: String::from(DEFAULT_ANIMATION_COLOR),
        }
    }
}

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GroupID {
    Node(usize),
    Box(usize),
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

pub type NodeSet = (Node, Vec<usize>);

pub struct DiagramCore {
    pub el_ops: Vec<ElementOpt>,
    pub nodes: Vec<NodeSet>,
    pub boxes: Vec<Node>,
    pub links: Vec<LinkContainer>,
    pub idx: ScreenIndex,
    pub render_ops: DiagramOpt,
    pub center: Point,
    pending_updates: FxHashMap<ScreenSlot, IndexXY>,

    pub transform: Transform,
    pub groups: FxHashMap<u32, FxHashSet<GroupID>>, // group_id,set->node_ids
    pub img_cache: ImgCache,
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
    pub fn set_element_option(&self, id: u32, opt: ElementOpt) {
        self.core.borrow_mut().set_opt(id, opt);
    }
    pub fn set_data(
        &self,
        boxes: Vec<Node>,
        nodes: Vec<Node>,
        links: Vec<LinkSet>,
    ) -> Result<(), JsValue> {
        self.core.borrow_mut().set_data(boxes, nodes, links)
    }

    pub fn mount(&self, width: u32, height: u32, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        self.core.borrow().mount(width, height, canvas)
    }
    pub fn unmount(&self) {
        self.core.borrow().unmount();
    }
}

#[wasm_bindgen(inspectable)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LinkAndElement {
    pub link: usize,
    pub element: usize,
}

#[wasm_bindgen]
pub enum CoreMouseEvent {
    MouseOverLink(LinkAndElement),
    MouseOverBundle(LinkAndElement),
    MoseOverNode(usize),
    MoseOverBox(usize),
    TransForm(Transform),
    Moved(MovedElements),
}

#[wasm_bindgen(inspectable, getter_with_clone)]
pub struct MovedElements {
    pub nodes: Vec<NodeChanges>,
    pub boxes: Vec<NodeChanges>,
}
#[wasm_bindgen(inspectable)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeChanges {
    pub id: usize,
    pub layout: Square,
}
impl DiagramCore {
    pub fn contains_point(&self, p: &Point) -> LookupPointResult {
        self.idx.contains_point(p, self)
    }

    pub fn run_callback(&self, e: UiEvent, p: &Point) {
        let cb = match &self.render_ops.callback {
            Some(cb) => cb,
            None => return,
        };
        let event = match e {
            UiEvent::MouseOver(res) => match res {
                LookupPointResult::Node(id) => JsValue::from(CoreMouseEvent::MoseOverNode(id)),
                LookupPointResult::Box(id) => JsValue::from(CoreMouseEvent::MoseOverBox(id)),
                LookupPointResult::Bundle((link, element)) => {
                    JsValue::from(CoreMouseEvent::MouseOverBundle(LinkAndElement {
                        link,
                        element,
                    }))
                }
                LookupPointResult::Link((link, element)) => {
                    JsValue::from(CoreMouseEvent::MouseOverLink(LinkAndElement {
                        link,
                        element,
                    }))
                }

                _ => return,
            },
        };

        let _ = cb.call2(&JsValue::null(), &event, &JsValue::from(*p));
    }
    pub fn new(render_ops: DiagramOpt) -> Rc<RefCell<Self>> {
        let render = Render::new();
        let mut res = Self {
            nodes: Vec::new(),
            boxes: Vec::new(),
            links: Vec::new(),
            el_ops: vec![ElementOpt::defaults()],
            idx: ScreenIndex::new(render_ops.index_step),
            render_ops,
            center: ZERO_POINT,
            transform: ZERO_TRANSFORM,
            groups: FxHashMap::default(),
            img_cache: ImgCache::new(Rc::clone(&render)),
            pending_updates: FxHashMap::default(),
            render,
        };

        res.el_ops.insert(0, ElementOpt::defaults());
        let this = Rc::new(RefCell::new(res));
        this.borrow_mut().render.borrow_mut().diagram = Rc::downgrade(&this);

        this
    }

    pub fn get_opt(&self, id: usize) -> &ElementOpt {
        match self.el_ops.get(id) {
            Some(opt) => opt,
            None => &self.el_ops[0],
        }
    }

    pub fn get_highlights(&self, lookup: &LookupPointResult) -> HighlightTargets {
        let mut nodes = Vec::new();
        let mut links = Vec::new();
        let mut bundles = Vec::new();
        let mut boxes = Vec::new();
        match lookup {
            LookupPointResult::Link((idx, el)) => {
                links.push(LinkAndElement {
                    link: *idx,
                    element: *el,
                });
                let link = &self.links[*idx];
                nodes.reserve(2);
                nodes.push(link.ls.src);
                nodes.push(link.ls.dst);
            }
            LookupPointResult::Box(id) => {
                boxes.push(*id);
            }
            LookupPointResult::Node(id) => {
                nodes.push(*id);
            }
            LookupPointResult::Bundle((idx, el)) => {
                let link = &self.links[*idx];
                bundles.push(LinkAndElement {
                    link: *idx,
                    element: *el,
                });
                let bl = &link.ls.bundles[*idx].links;
                nodes.reserve(2);
                nodes.push(link.ls.src);
                nodes.push(link.ls.dst);
                links.reserve(bl.len());
                for el in bl {
                    links.push(LinkAndElement {
                        link: *idx,
                        element: *el,
                    });
                }
            }
            _ => (),
        }
        HighlightTargets {
            nodes,
            boxes,
            links,
            bundles,
        }
    }

    pub fn set_opt(&mut self, id: u32, opt: ElementOpt) {
        let id = id as usize;
        self.el_ops[id] = opt;
    }

    pub fn set_transform(&mut self, t: Transform) {
        self.transform = t;
    }
    pub fn get_transform(&self) -> Transform {
        self.transform
    }
    pub fn set_element_options(&mut self, el_ops: Vec<ElementOpt>) {
        if el_ops.len() == 0 {
            self.el_ops = vec![ElementOpt::defaults()];
        } else {
            self.el_ops = el_ops
        }
    }

    fn add_node(&mut self, id: usize, as_node: bool, node: &Node) {
        self.center.x += node.layout.x;
        self.center.y += node.layout.x;

        self.update_groups(&node.groups, id, as_node);
        let points = node.layout.idx(self.render_ops.index_step);
        let ss = match as_node {
            true => ScreenSlot::Node(id),
            false => ScreenSlot::Box(id),
        };
        self.idx.manage(&ss, points, IdxBoxAction::Add);
    }
    pub fn set_data(
        &mut self,
        boxes: Vec<Node>,
        nodes: Vec<Node>,
        links: Vec<LinkSet>,
    ) -> Result<(), JsValue> {
        self.clear();
        self.render.borrow().clear()?;
        self.boxes.reserve(boxes.len());
        for (id, node) in boxes.into_iter().enumerate() {
            self.add_node(id, false, &node);
            self.boxes.push(node);
        }
        self.nodes.reserve(nodes.len());
        for (id, node) in nodes.into_iter().enumerate() {
            self.add_node(id, true, &node);
            self.nodes.push((node, Vec::new()));
        }
        self.links.reserve(links.len());
        for lc in links {
            self.add_link(lc)?;
        }

        Ok(())
    }

    fn add_link(&mut self, ls: LinkSet) -> Result<usize, JsValue> {
        if ls.links.len() == 0 {
            return Err(JsValue::from(LINK_ADD_ERROR));
        }
        let id = self.links.len();
        let (lc, a, b);
        {
            (a, b) = (ls.src as usize, ls.dst as usize);
            if a == b {
                return Err(JsValue::from(LINK_ADD_ERROR));
            }
            let ((src, _), (dst, _)) = match (self.nodes.get(a), self.nodes.get(b)) {
                (Some(a), Some(b)) => (a, b),
                _ => return Err(JsValue::from(LINK_ADD_ERROR)),
            };

            lc = LinkContainer::new(ls, src, dst, &self.render_ops, id);
        }
        self.nodes[a].1.push(id);
        self.nodes[b].1.push(id);
        let points = lc.draw_data.index.idx(self.idx.step);
        self.idx
            .manage(&ScreenSlot::Link(id), points, IdxBoxAction::Add);
        self.links.push(lc);
        Ok(id)
    }

    fn update_groups(&mut self, groups: &Vec<u32>, id: usize, as_node: bool) {
        for group in groups {
            let set = match self.groups.get_mut(group) {
                Some(s) => s,
                None => {
                    let s = FxHashSet::default();
                    self.groups.insert(*group, s);
                    unsafe { self.groups.get_mut(group).unwrap_unchecked() }
                }
            };

            set.insert(match as_node {
                true => GroupID::Node(id),
                false => GroupID::Box(id),
            });
        }
    }
    fn clear(&mut self) {
        self.nodes.clear();
        self.boxes.clear();
        self.links.clear();
        self.idx.clear();
        self.groups.clear();
        self.center = ZERO_POINT;
    }

    pub fn get_related_nodes(&self, node_ids: &[GroupID]) -> Vec<GroupID> {
        let mut ids = FxHashSet::default();
        ids.reserve(node_ids.len());
        for group in node_ids {
            ids.insert(*group);
            let node = match group {
                GroupID::Box(id) => &self.boxes[*id],
                GroupID::Node(id) => &self.nodes[*id].0,
            };
            for gid in &node.groups {
                let group = unsafe { self.groups.get(gid).unwrap_unchecked() };
                for node_id in group {
                    ids.insert(*node_id);
                }
            }
        }
        ids.into_iter().collect()
    }

    pub fn move_nodes(&mut self, distance: &Point, node_ids: &[GroupID]) {
        let mut links = FxHashSet::default();
        let step = self.idx.step;
        self.center.x += distance.x * node_ids.len() as f64;
        self.center.y += distance.y * node_ids.len() as f64;

        for group in node_ids {
            let (node, ss) = match group {
                GroupID::Box(box_id) => (&mut self.boxes[*box_id], ScreenSlot::Box(*box_id)),
                GroupID::Node(node_id) => {
                    for lid in &self.nodes[*node_id].1 {
                        links.insert(*lid);
                    }
                    (&mut self.nodes[*node_id].0, ScreenSlot::Node(*node_id))
                }
            };
            if !self.pending_updates.contains_key(&ss) {
                self.pending_updates.insert(ss, node.layout.idx(step));
            }
            node.layout.move_distance(distance);
        }

        for lid in links {
            let lc = unsafe { self.links.get_mut(lid).unwrap_unchecked() };
            let ss = ScreenSlot::Link(lid);

            if !self.pending_updates.contains_key(&ss) {
                self.pending_updates
                    .insert(ss, lc.draw_data.index.idx(step));
            }
            lc.draw_data.move_distance(distance);
        }
    }
    pub fn finish_move(&mut self) {
        let mut keys = Vec::with_capacity(self.pending_updates.len());
        for id in self.pending_updates.keys() {
            keys.push(*id);
        }
        keys.sort();
        let step = self.idx.step;
        for ss in keys {
            let old = unsafe { self.pending_updates.remove(&ss).unwrap_unchecked() };
            match ss {
                ScreenSlot::Box(b) => {
                    let new = self.boxes[b].layout.idx(step);
                    self.idx.update(&ScreenSlot::Box(b), old, new);
                }
                ScreenSlot::Node(b) => {
                    let new = self.nodes[b].0.layout.idx(step);
                    self.idx.update(&ScreenSlot::Node(b), old, new);
                }
                ScreenSlot::Link(b) => {
                    let new = self.links[b].draw_data.index.idx(step);
                    self.idx.update(&ScreenSlot::Link(b), old, new);
                }
            }
        }
    }

    pub fn mount(&self, width: u32, height: u32, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        self.render.borrow_mut().mount(width, height, canvas)
    }

    pub fn unmount(&self) {
        self.render.borrow_mut().unmount();
    }
}
