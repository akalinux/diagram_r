use rustc_hash::{FxHashMap, FxHashSet};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use web_sys::HtmlCanvasElement;

use crate::{
    DiagramOpt, ElementOpt, Point, Transform,
    bsp::{IndexXY, LookupPointResult, ScreenIndex, ScreenSlot, iter::IdxBoxAction},
    constants::*,
    imgcache::ImgCache,
    link::{LinkContainer, LinkSet},
    node::Node,
    render::{CoreRender, pointerwatcher::PointerWatcher, timeout::Timeout},
    square::Square,
    utils::to_map_xy,
};
use wasm_bindgen::prelude::*;

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

#[cfg(not(feature = "webgl"))]
fn build_render(
    canvas: &HtmlCanvasElement,
    diagram: Weak<RefCell<DiagramCore>>,
) -> Result<Box<dyn CoreRender>, JsValue> {
    use crate::render::{BuildRender, canvasrender::CanvasRender};

    CanvasRender::new(canvas, diagram)
}

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GroupID {
    Node(usize),
    Box(usize),
}

pub type NodeSet = (Node, Vec<usize>);

pub struct DiagramCore {
    pub this: Weak<RefCell<Self>>,
    pub el_ops: Vec<ElementOpt>,
    pub nodes: RefCell<Vec<NodeSet>>,
    pub boxes: RefCell<Vec<Node>>,
    pub links: RefCell<Vec<LinkContainer>>,
    pub idx: RefCell<ScreenIndex>,
    pub render_ops: DiagramOpt,
    pub center: RefCell<Point>,
    pub pending_updates: RefCell<FxHashMap<ScreenSlot, IndexXY>>,

    pub transform: RefCell<Transform>,
    pub groups: RefCell<FxHashMap<u32, FxHashSet<GroupID>>>, // group_id,set->node_ids
    pub img_cache: ImgCache,
    pub render: RefCell<Option<Box<dyn CoreRender>>>,
    pub timeout: RefCell<Option<Timeout>>,
    pub current_target: RefCell<Option<(LookupPointResult, Point)>>,
    pub highlights: RefCell<Option<HighlightTargets>>,
    pub watcher: RefCell<Option<PointerWatcher>>,
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
        self.core.borrow().set_transform(t);
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

    pub fn mount(&self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        self.core.borrow().mount(canvas)
    }
    pub fn unmount(&self) {
        self.core.borrow().unmount();
    }
    pub fn render(&self) -> Result<(), JsValue> {
        self.core.borrow().render()
    }
}

#[wasm_bindgen(inspectable)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LinkAndElement {
    pub link: usize,
    pub element: usize,
}

impl LinkAndElement {
    pub fn new(link: usize, element: usize) -> Self {
        Self { link, element }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct HighlightTargets {
    pub nodes: Vec<usize>,
    pub boxes: Vec<usize>,
    pub links: Vec<LinkAndElement>,
    pub bundles: Vec<LinkAndElement>,
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
        self.idx.borrow().contains_point(p, self)
    }

    pub fn run_callback(&self, event: CoreMouseEvent, p: &Point) {
        let cb = match &self.render_ops.callback {
            Some(cb) => cb,
            None => return,
        };

        let _ = cb.call2(&JsValue::null(), &JsValue::from(event), &JsValue::from(*p));
    }
    pub fn new(render_ops: DiagramOpt) -> Rc<RefCell<Self>> {
        let mut res = Self {
            timeout: RefCell::new(None),
            current_target: RefCell::new(None),
            highlights: RefCell::new(None),
            this: Weak::new(),
            nodes: RefCell::new(Vec::new()),
            boxes: RefCell::new(Vec::new()),
            links: RefCell::new(Vec::new()),
            el_ops: vec![ElementOpt::defaults()],
            idx: RefCell::new(ScreenIndex::new(render_ops.index_step)),
            render_ops,
            center: RefCell::new(ZERO_POINT),
            transform: RefCell::new(ZERO_TRANSFORM),
            groups: RefCell::new(FxHashMap::default()),
            img_cache: ImgCache::new(Weak::new()),
            pending_updates: RefCell::new(FxHashMap::default()),
            render: RefCell::new(None),
            watcher: RefCell::new(None),
        };

        res.el_ops.insert(0, ElementOpt::defaults());
        let this = Rc::new(RefCell::new(res));
        this.borrow_mut().this = Rc::downgrade(&this);
        this.borrow_mut().img_cache.diagram = Rc::downgrade(&this);

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
                let link = &self.links.borrow()[*idx];
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
                let link = &self.links.borrow()[*idx];
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

    pub fn set_transform(&self, t: Transform) {
        self.transform.replace(t);
    }
    pub fn get_transform(&self) -> Transform {
        *self.transform.borrow()
    }
    pub fn set_element_options(&mut self, el_ops: Vec<ElementOpt>) {
        if el_ops.len() == 0 {
            self.el_ops = vec![ElementOpt::defaults()];
        } else {
            self.el_ops = el_ops
        }
    }

    fn add_node(&self, id: usize, as_node: bool, node: &Node) {
        let mut center = self.center.borrow_mut();
        center.x += node.layout.x;
        center.y += node.layout.x;

        self.update_groups(&node.groups, id, as_node);
        let points = node.layout.idx(self.render_ops.index_step);
        let ss = match as_node {
            true => ScreenSlot::Node(id),
            false => ScreenSlot::Box(id),
        };
        self.idx.borrow_mut().manage(&ss, points, IdxBoxAction::Add);
    }
    pub fn set_data(
        &mut self,
        boxes: Vec<Node>,
        nodes: Vec<Node>,
        links: Vec<LinkSet>,
    ) -> Result<(), JsValue> {
        self.clear();
        self.boxes.borrow_mut().reserve(boxes.len());
        for (id, node) in boxes.into_iter().enumerate() {
            self.add_node(id, false, &node);
            self.boxes.borrow_mut().push(node);
        }
        self.nodes.borrow_mut().reserve(nodes.len());
        for (id, node) in nodes.into_iter().enumerate() {
            self.add_node(id, true, &node);
            self.nodes.borrow_mut().push((node, Vec::new()));
        }
        self.links.borrow_mut().reserve(links.len());
        for lc in links {
            self.add_link(lc)?;
        }

        Ok(())
    }

    fn add_link(&self, ls: LinkSet) -> Result<usize, JsValue> {
        if ls.links.len() == 0 {
            return Err(JsValue::from(LINK_ADD_ERROR));
        }
        let id = self.links.borrow().len();
        let (lc, a, b);
        {
            (a, b) = (ls.src as usize, ls.dst as usize);
            if a == b {
                return Err(JsValue::from(LINK_ADD_ERROR));
            }
            let nodes = self.nodes.borrow();
            let ((src, _), (dst, _)) = match (nodes.get(a), nodes.get(b)) {
                (Some(a), Some(b)) => (a, b),
                _ => return Err(JsValue::from(LINK_ADD_ERROR)),
            };

            lc = LinkContainer::new(ls, src, dst, &self.render_ops, id);
        }
        self.nodes.borrow_mut()[a].1.push(id);
        self.nodes.borrow_mut()[b].1.push(id);
        let points = lc.draw_data.index.idx(self.render_ops.index_step);
        self.idx
            .borrow_mut()
            .manage(&ScreenSlot::Link(id), points, IdxBoxAction::Add);
        self.links.borrow_mut().push(lc);
        Ok(id)
    }

    fn update_groups(&self, groups: &Vec<u32>, id: usize, as_node: bool) {
        let mut grps = self.groups.borrow_mut();
        for group in groups {
            let set = match grps.get_mut(group) {
                Some(s) => s,
                None => {
                    let s = FxHashSet::default();
                    grps.insert(*group, s);
                    unsafe { grps.get_mut(group).unwrap_unchecked() }
                }
            };

            set.insert(match as_node {
                true => GroupID::Node(id),
                false => GroupID::Box(id),
            });
        }
    }
    fn clear(&mut self) {
        self.nodes.borrow_mut().clear();
        self.boxes.borrow_mut().clear();
        self.links.borrow_mut().clear();
        self.idx.borrow_mut().clear();
        self.groups.borrow_mut().clear();
        self.center.replace(ZERO_POINT);
        self.clear_render();
    }

    pub fn get_related_nodes(&self, node_ids: &[GroupID]) -> Vec<GroupID> {
        let mut ids = FxHashSet::default();
        let boxes = self.boxes.borrow();
        let nodes = self.nodes.borrow();
        let groups = self.groups.borrow();
        ids.reserve(node_ids.len());
        for group in node_ids {
            ids.insert(*group);
            let node = match group {
                GroupID::Box(id) => &boxes[*id],
                GroupID::Node(id) => &nodes[*id].0,
            };
            for gid in &node.groups {
                let group = unsafe { groups.get(gid).unwrap_unchecked() };
                for node_id in group {
                    ids.insert(*node_id);
                }
            }
        }
        ids.into_iter().collect()
    }

    pub fn on_img(&self, cache: &ImgCache) {
        match self.render_ops.bulk_img_update {
            true => {
                if cache.is_done() {
                    let _ = self.render();
                    return;
                }
            }
            false => {
                let _ = self.render();
            }
        };
    }
    pub fn move_nodes(&self, distance: &Point, node_ids: &[GroupID]) {
        let mut links = FxHashSet::default();
        let step = self.render_ops.index_step;
        {
            let mut center = self.center.borrow_mut();
            center.x += distance.x * node_ids.len() as f32;
            center.y += distance.y * node_ids.len() as f32;
        }

        for group in node_ids {
            let (node, ss) = match group {
                GroupID::Box(box_id) => (
                    &mut self.boxes.borrow_mut()[*box_id],
                    ScreenSlot::Box(*box_id),
                ),
                GroupID::Node(node_id) => {
                    for lid in &self.nodes.borrow_mut()[*node_id].1 {
                        links.insert(*lid);
                    }
                    (
                        &mut self.nodes.borrow_mut()[*node_id].0,
                        ScreenSlot::Node(*node_id),
                    )
                }
            };
            if !self.pending_updates.borrow().contains_key(&ss) {
                self.pending_updates
                    .borrow_mut()
                    .insert(ss, node.layout.idx(step));
            }
            node.layout.move_distance(distance);
        }

        let mut all_links = self.links.borrow_mut();
        for lid in links {
            let lc = &mut all_links[lid];
            let ss = ScreenSlot::Link(lid);

            if !self.pending_updates.borrow().contains_key(&ss) {
                self.pending_updates
                    .borrow_mut()
                    .insert(ss, lc.draw_data.index.idx(step));
            }
            lc.draw_data.move_distance(distance);
            self.update_render(ScreenSlot::Link(lid), distance);
        }
    }
    fn update_render(&self, target: ScreenSlot, distance: &Point) {
        let rs = self.render.borrow();
        match rs.as_ref() {
            Some(render) => render.as_ref().update(target, distance),
            None => (),
        }
    }
    pub fn finish_move(&self) {
        let mut keys = Vec::with_capacity(self.pending_updates.borrow().len());
        for id in self.pending_updates.borrow().keys() {
            keys.push(*id);
        }
        keys.sort();
        let step = self.render_ops.index_step;
        let mut idx = self.idx.borrow_mut();
        let nodes = self.nodes.borrow();
        let boxes = self.boxes.borrow();
        let links = &self.links.borrow();
        for ss in keys {
            let old = unsafe {
                self.pending_updates
                    .borrow_mut()
                    .remove(&ss)
                    .unwrap_unchecked()
            };
            match ss {
                ScreenSlot::Box(b) => {
                    let new = boxes[b].layout.idx(step);
                    idx.update(&ScreenSlot::Box(b), old, new);
                }
                ScreenSlot::Node(b) => {
                    let new = nodes[b].0.layout.idx(step);
                    idx.update(&ScreenSlot::Node(b), old, new);
                }
                ScreenSlot::Link(b) => {
                    let new = links[b].draw_data.index.idx(step);
                    idx.update(&ScreenSlot::Link(b), old, new);
                }
            }
        }
    }

    pub fn mount(&self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        let render = build_render(&canvas, self.this.clone())?;
        if self.render_ops.interactive {
            let watcher = PointerWatcher::new(self.this.clone(), canvas)?;
            self.watcher.replace(Some(watcher));
        }
        self.render.replace(Some(render));
        Ok(())
    }

    fn clear_render(&self) {
        match self.render.borrow().as_ref() {
            Some(r) => r.clear(),
            _ => (),
        }
    }
    pub fn unmount(&self) {
        self.clear_render();
    }
}

// Pointer event code is here
impl DiagramCore {
    fn render(&self) -> Result<(), JsValue> {
        Ok(())
    }

    fn clear_timeout(&self) {
        self.timeout.replace(None);
    }
    pub fn to_map_xy(&self, p: &Point) -> Point {
        to_map_xy(p, &*self.transform.borrow())
    }
    fn set_timeout(&self) {
        let this = unsafe { self.this.upgrade().unwrap_unchecked() };

        let job = move || {
            match this.borrow().current_target.replace(None) {
                Some((l, p)) => {
                    match l {
                        LookupPointResult::NoMatch => {
                            let np = this.borrow().to_map_xy(&p);
                            let event = match this.borrow().contains_point(&np) {
                                LookupPointResult::Box(id) => CoreMouseEvent::MoseOverBox(id),
                                LookupPointResult::Node(id) => CoreMouseEvent::MoseOverNode(id),
                                LookupPointResult::Link(id) => {
                                    CoreMouseEvent::MouseOverLink(LinkAndElement::new(id.0, id.1))
                                }
                                LookupPointResult::Bundle(id) => {
                                    CoreMouseEvent::MouseOverBundle(LinkAndElement::new(id.0, id.1))
                                }

                                _ => {
                                    this.borrow().current_target.replace(Some((l, p)));
                                    return;
                                }
                            };
                            this.borrow().run_callback(event, &p);
                        }
                        _ => {
                            this.borrow().current_target.replace(Some((l, p)));
                            return;
                        }
                    };
                }
                None => (),
            };
        };
        match Timeout::new(job, self.render_ops.timeout) {
            Err(_) => return,
            Ok(t) => {
                self.timeout.replace(Some(t));
            }
        };
    }

    pub fn on_mouse_down(&self, p: &Point) {
        self.clear_timeout();
        let lp = self.to_map_xy(p);
        let res = self.contains_point(&lp);

        // grab the center of what ever was clicked on
        match &res {
            LookupPointResult::NoMatch => {
                // screen is always a synthetic fall through
                self.current_target
                    .replace(Some((LookupPointResult::Screen, *p)));
                return;
            }
            _ => (),
        };

        self.current_target.replace(Some((res, *p)));
    }
    pub fn on_mouse_up(&self, p: &Point) {
        self.move_lookup(p);
    }
    pub fn on_mouse_enter(&self, p: &Point) {
        self.current_target
            .replace(Some((LookupPointResult::NoMatch, *p)));
    }
    pub fn on_mouse_leave(&self, _: &Point) {
        self.clear_timeout();
        self.current_target.replace(None);
        self.highlights.replace(None);
    }
    fn move_lookup(&self, p: &Point) {
        match self.current_target.borrow_mut().as_mut() {
            Some((l, op)) => {
                let nodes = match l {
                    LookupPointResult::NoMatch => {
                        *op = *p;
                        self.set_timeout();
                        return;
                    }
                    LookupPointResult::Screen => {
                        let distance = self.to_map_xy(op).get_move_distance(p);
                        *op = *p;
                        let mut t = self.get_transform();

                        t.x += distance.x;
                        t.y += distance.y;

                        self.set_transform(t);
                        let _ = self.render();
                        return;
                    }
                    LookupPointResult::Box(id) => self.get_related_nodes(&[GroupID::Box(*id)]),
                    LookupPointResult::Node(id) => self.get_related_nodes(&[GroupID::Node(*id)]),
                    LookupPointResult::Bundle((link_id, _))
                    | LookupPointResult::Link((link_id, _)) => {
                        let link = &self.links.borrow()[*link_id].ls;
                        vec![GroupID::Node(link.src), GroupID::Node(link.dst)]
                    }
                };
                let distance = self.to_map_xy(op).get_move_distance(p);
                self.move_nodes(&distance, &nodes);
                *op = *p;
                let _ = self.render();
                return;
            }
            None => (),
        }
        // if we got here.. then we need to setup for timeout
        self.current_target
            .replace(Some((LookupPointResult::NoMatch, *p)));
        self.set_timeout();
    }
    pub fn on_mouse_move(&self, p: &Point) {
        self.clear_timeout();
        self.highlights.replace(None);
        self.move_lookup(p);
    }

    pub fn on_mouse_wheel(&self, delta: f64) {
        let mut t = self.get_transform();
        t.k -= match delta < 0.0 {
            true => self.render_ops.wheel_move,
            false => -self.render_ops.wheel_move,
        };
        if t.k < 0.0 {
            t.k = self.render_ops.wheel_move;
        }
        self.set_transform(t);
        let _ = self.render();
    }
}
