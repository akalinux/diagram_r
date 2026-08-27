use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use std::{
    cell::RefCell,
    mem,
    rc::{Rc, Weak},
};
use web_sys::HtmlCanvasElement;

use crate::{
    DiagramOpt, ElementOpt, GridOpt, Point, Transform,
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
    canvas: HtmlCanvasElement,
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

#[derive(Clone, Debug)]
pub enum CurrentTarget {
    Move(Vec<MoveTarget>, Point),
    Screen(Point),
    Lookup(Point),
    Highlight,
    None,
}

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MoveTarget {
    Node(usize),
    Box(usize),
    Link(usize),
}

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
    pub img_cache: ImgCache,
    pub render: RefCell<Option<Box<dyn CoreRender>>>,
    pub timeout: RefCell<Option<Timeout>>,
    pub current_target: RefCell<CurrentTarget>,
    pub highlights: RefCell<Option<HighlightTargets>>,
    pub watcher: RefCell<Option<PointerWatcher>>,
}

#[wasm_bindgen]
pub struct Diagram {
    core: Rc<RefCell<DiagramCore>>,
}

#[wasm_bindgen]
impl Diagram {
    #[wasm_bindgen(constructor)]
    pub fn new(render_ops: DiagramOpt) -> Self {
        Self {
            core: DiagramCore::new(render_ops),
        }
    }
    pub fn set_grid_opts(&self, ops: Option<GridOpt>) {
        self.core.borrow_mut().set_grid_opts(ops);
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
    pub arc: Option<usize>,
}

#[wasm_bindgen]
pub enum CoreMouseEvent {
    MouseOverLink(LinkAndElement),
    MouseOverBundle(LinkAndElement),
    MoseOverNode(usize),
    MouseOverArc(usize),
    MoseOverBox(usize),
    TransForm(Transform),
    Moved(MovedElements),
}

#[wasm_bindgen(inspectable, getter_with_clone)]
pub struct MovedElements {
    pub nodes: Vec<NodeChanges>,
    pub boxes: Vec<NodeChanges>,
    pub links: Vec<LinkChanges>,
}
#[wasm_bindgen(inspectable)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeChanges {
    pub id: usize,
    pub layout: Square,
}

#[wasm_bindgen(inspectable)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkChanges {
    id: usize,
    point: Point,
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
            current_target: RefCell::new(CurrentTarget::None),
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
        let mut arc = None;
        match lookup {
            LookupPointResult::Arc(id) => arc = Some(*id),
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
                let bl = &link.ls.bundles[*el].links;
                nodes.reserve(2);
                nodes.push(link.ls.src);
                nodes.push(link.ls.dst);
                links.reserve(bl.len());
                let src = &link.ls.links;
                for el in bl {
                    match src.get(*el) {
                        Some(_) => {
                            links.push(LinkAndElement {
                                link: *idx,
                                element: *el,
                            });
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
        HighlightTargets {
            nodes,
            boxes,
            links,
            bundles,
            arc,
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
        self.img_cache.load_images(&self.el_ops);
    }

    fn add_node(&self, id: usize, as_node: bool, node: &Node) {
        let mut center = self.center.borrow_mut();
        center.x += node.layout.x;
        center.y += node.layout.x;

        let points = node.layout.idx(self.render_ops.index_step);
        let ss = match as_node {
            true => ScreenSlot::Node(id),
            false => ScreenSlot::Box(id),
        };
        if self.render_ops.interactive {
            self.idx.borrow_mut().manage(&ss, points, IdxBoxAction::Add);
        }
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

    pub fn get_link_src_dst<'n>(&self, id: usize) -> Option<(&Node, &Node)> {
        let links = self.links.borrow();
        let lc = match links.get(id) {
            Some(l) => l,
            None => return None,
        };
        let nodes = self.nodes.borrow();
        let (src, dst) = (lc.ls.src, lc.ls.dst);
        match (nodes.get(src), nodes.get(dst)) {
            (Some(a), Some(b)) => unsafe { mem::transmute(Some((&a.0, &b.0))) },
            _ => None,
        }
    }
    pub fn link_src_dst(&self, id: usize) -> (&Node, &Node) {
        unsafe { self.get_link_src_dst(id).unwrap_unchecked() }
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
        if self.render_ops.interactive {
            self.nodes.borrow_mut()[a].1.push(id);
            self.nodes.borrow_mut()[b].1.push(id);
            let points = lc.draw_data.index.idx(self.render_ops.index_step);
            self.idx
                .borrow_mut()
                .manage(&ScreenSlot::Link(id), points, IdxBoxAction::Add);
        }
        self.links.borrow_mut().push(lc);
        Ok(id)
    }

    fn clear(&mut self) {
        self.nodes.borrow_mut().clear();
        self.boxes.borrow_mut().clear();
        self.links.borrow_mut().clear();
        self.idx.borrow_mut().clear();
        self.center.replace(ZERO_POINT);
        self.clear_render();
    }

    pub fn get_related_nodes(&self, node_ids: &[GroupID]) -> Vec<MoveTarget> {
        let mut ids = FxHashSet::with_capacity_and_hasher(node_ids.len(), FxBuildHasher::default());
        let boxes = self.boxes.borrow();
        let nodes = self.nodes.borrow();

        for group in node_ids {
            let node = match group {
                GroupID::Box(id) => {
                    ids.insert(MoveTarget::Box(*id));
                    &boxes[*id]
                }
                GroupID::Node(id) => {
                    ids.insert(MoveTarget::Node(*id));
                    &nodes[*id].0
                }
            };
            for id in &node.nodes {
                // defensive code.. peope do people things..
                if nodes.get(*id).is_some() {
                    ids.insert(MoveTarget::Node(*id));
                }
            }
            for id in &node.boxes {
                // defensive code.. peope do people things..
                if boxes.get(*id).is_some() {
                    ids.insert(MoveTarget::Box(*id));
                }
            }
        }
        ids.into_iter().collect()
    }

    pub fn on_img(&self, cache: &ImgCache) {
        if cache.is_done() {
            let _ = self.render();
        }
    }
    pub fn move_targets(&self, distance: &Point, node_ids: &[MoveTarget]) {
        let mut links = FxHashSet::default();
        let mut upodated_nodes = FxHashSet::default();
        upodated_nodes.reserve(node_ids.len());
        let step = self.render_ops.index_step;
        {
            let mut center = self.center.borrow_mut();
            center.x += distance.x * node_ids.len() as f32;
            center.y += distance.y * node_ids.len() as f32;
        }

        for group in node_ids {
            let (node, ss) = match group {
                MoveTarget::Box(box_id) => (
                    &mut self.boxes.borrow_mut()[*box_id],
                    ScreenSlot::Box(*box_id),
                ),
                MoveTarget::Node(node_id) => {
                    upodated_nodes.insert(*node_id);
                    for lid in &self.nodes.borrow_mut()[*node_id].1 {
                        links.insert(*lid);
                    }
                    (
                        &mut self.nodes.borrow_mut()[*node_id].0,
                        ScreenSlot::Node(*node_id),
                    )
                }
                MoveTarget::Link(id) => {
                    let ss = ScreenSlot::Link(*id);

                    if !self.pending_updates.borrow().contains_key(&ss) {
                        let link = &self.links.borrow_mut()[*id];

                        self.pending_updates
                            .borrow_mut()
                            .insert(ss, link.draw_data.index.idx(step));
                    }
                    let (src, dst) = self.link_src_dst(*id);
                    let link = &mut self.links.borrow_mut()[*id];
                    link.move_arc(distance, src, dst, &self.render_ops);
                    continue;
                }
            };
            if !self.pending_updates.borrow().contains_key(&ss) {
                self.pending_updates
                    .borrow_mut()
                    .insert(ss, node.layout.idx(step));
            }
            node.layout.move_distance(distance);
            self.update_render(ss);
        }

        let mut all_links = self.links.borrow_mut();
        let nodes = self.nodes.borrow();
        let (mut lc, mut ss);
        for lid in links {
            lc = &mut all_links[lid];
            ss = ScreenSlot::Link(lid);

            if !self.pending_updates.borrow().contains_key(&ss) {
                self.pending_updates
                    .borrow_mut()
                    .insert(ss, lc.draw_data.index.idx(step));
            }
            let (src, dst) = (lc.ls.src, lc.ls.dst);
            if upodated_nodes.contains(&src) && upodated_nodes.contains(&dst) {
                lc.move_distance(distance);
            } else {
                lc.draw_data =
                    lc.ls
                        .build_draw_data(&nodes[src].0, &nodes[dst].0, &self.render_ops);
            }
            self.update_render(ScreenSlot::Link(lid));
        }
    }
    fn update_render(&self, target: ScreenSlot) {
        let rs = self.render.borrow();
        match rs.as_ref() {
            Some(render) => render.as_ref().update(target),
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
        let render = build_render(canvas.clone(), self.this.clone())?;
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
        self.render.replace(None);
        self.watcher.replace(None);
    }
}

// Pointer event code is here
impl DiagramCore {
    pub fn set_grid_opts(&mut self, ops: Option<GridOpt>) {
        self.render_ops.grid_opt = ops;
    }
    pub fn render(&self) -> Result<(), JsValue> {
        match (self.img_cache.is_done(), self.render.borrow().as_ref()) {
            (true, Some(r)) => r.render(),
            _ => Ok(()),
        }
    }
    pub fn animate(&self) -> Result<(), JsValue> {
        match (self.img_cache.is_done(), self.render.borrow().as_ref()) {
            (true, Some(r)) => r.animate(),
            _ => Ok(()),
        }
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
            let core = this.borrow();
            let mut check = core.current_target.borrow_mut();
            match &mut *check {
                CurrentTarget::Lookup(p) => {
                    let np = this.borrow().to_map_xy(p);
                    let res = this.borrow().contains_point(&np);
                    let event = match &res {
                        LookupPointResult::Arc(id) => CoreMouseEvent::MouseOverArc(*id),
                        LookupPointResult::Box(id) => CoreMouseEvent::MoseOverBox(*id),
                        LookupPointResult::Node(id) => CoreMouseEvent::MoseOverNode(*id),
                        LookupPointResult::Link(id) => {
                            CoreMouseEvent::MouseOverLink(LinkAndElement::new(id.0, id.1))
                        }
                        LookupPointResult::Bundle(id) => {
                            CoreMouseEvent::MouseOverBundle(LinkAndElement::new(id.0, id.1))
                        }
                        _ => {
                            *check = CurrentTarget::None;
                            return;
                        }
                    };
                    let higlights = this.borrow().get_highlights(&res);
                    this.borrow().highlights.replace(Some(higlights));
                    let _ = this.borrow().render();
                    this.borrow().run_callback(event, p);
                    *check = CurrentTarget::Highlight;
                    return;
                }
                _ => {
                    return;
                }
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

        self.current_target
            .replace(self.get_current_target(&res, p));
    }
    pub fn on_mouse_up(&self, p: &Point) {
        // still need to update the index
        if self.move_current_target(p) {
            self.finish_move();
        } else {
            return;
        }
        let res = self.current_target.replace(CurrentTarget::Lookup(*p));

        self.set_timeout();
        if let CurrentTarget::Move(g, _) = res {
            let mut nodes = Vec::new();
            let mut boxes = Vec::new();
            let mut links = Vec::new();
            for o in g {
                match o {
                    MoveTarget::Box(b) => boxes.push(NodeChanges {
                        id: b,
                        layout: self.boxes.borrow()[b].layout,
                    }),
                    MoveTarget::Node(b) => nodes.push(NodeChanges {
                        id: b,
                        layout: self.nodes.borrow()[b].0.layout,
                    }),
                    MoveTarget::Link(id) => links.push(LinkChanges {
                        id,
                        point: unsafe {
                            let links = self.links.borrow();
                            let p = links[id].ls.point.as_ref().unwrap_unchecked();
                            let res = *&p.point;
                            res
                        },
                    }),
                }
            }
            let moved = MovedElements {
                nodes,
                boxes,
                links,
            };
            self.run_callback(CoreMouseEvent::Moved(moved), p);
        }
    }

    fn interaction_reset(&self) {
        self.clear_timeout();
        self.current_target.replace(CurrentTarget::None);
        self.highlights.replace(None);
    }

    pub fn on_mouse_enter(&self, _: &Point) {
        self.interaction_reset()
    }

    pub fn on_mouse_leave(&self, _: &Point) {
        self.interaction_reset();
    }

    fn get_current_target(&self, lookup: &LookupPointResult, p: &Point) -> CurrentTarget {
        CurrentTarget::Move(
            match lookup {
                LookupPointResult::NoMatch => {
                    return CurrentTarget::Screen(*p);
                }
                LookupPointResult::Arc(id) => vec![MoveTarget::Link(*id)],
                LookupPointResult::Box(id) => self.get_related_nodes(&[GroupID::Box(*id)]),
                LookupPointResult::Node(id) => self.get_related_nodes(&[GroupID::Node(*id)]),
                LookupPointResult::Bundle((link_id, _)) | LookupPointResult::Link((link_id, _)) => {
                    let link = &self.links.borrow()[*link_id].ls;
                    vec![MoveTarget::Node(link.src), MoveTarget::Node(link.dst)]
                }
            },
            *p,
        )
    }
    /// Moves the current targets
    fn move_current_target(&self, p: &Point) -> bool {
        let mut check = self.current_target.borrow_mut();
        self.highlights.replace(None);
        let (nodes, op) = match &mut *check {
            CurrentTarget::Highlight => {
                *check = CurrentTarget::Lookup(*p);
                self.set_timeout();
                return true;
            }
            CurrentTarget::None => {
                // in this case we need to transition from none to our current lookup
                *check = CurrentTarget::Lookup(*p);
                self.set_timeout();
                return false;
            }
            CurrentTarget::Screen(op) => {
                let distance = op.get_move_distance(p);
                *op = *p;
                {
                    let mut t = self.transform.borrow_mut();
                    t.x += distance.x;
                    t.y += distance.y;
                }
                self.clear_timeout();
                let _ = self.render();
                return true;
            }
            CurrentTarget::Move(nodes, op) => (nodes, op),
            CurrentTarget::Lookup(op) => {
                *op = *p;
                self.set_timeout();
                return false;
            }
        };
        self.clear_timeout();
        let distance = &op
            .get_move_distance(p)
            .scale(1.0 / self.transform.borrow().k);
        self.move_targets(&distance, nodes);
        *op = *p;
        let _ = self.render();
        true
    }
    pub fn on_mouse_move(&self, p: &Point) {
        if self.move_current_target(p) {
            let _ = self.render();
        }
    }

    fn get_width_height(&self) -> (f32, f32) {
        unsafe {
            self.render
                .borrow()
                .as_ref()
                .unwrap_unchecked()
                .get_width_height()
        }
    }
    pub fn on_mouse_wheel(&self, p: &Point, delta: f64) {
        self.clear_timeout();
        let t = {
            let mut t = self.get_transform();

            let mut offset = match delta < 0.0 {
                true => self.render_ops.wheel_move,
                false => -self.render_ops.wheel_move,
            };
            t.k += offset;
            if t.k < self.render_ops.min_k || t.k > self.render_ops.max_k {
                return;
            }
            let (width, height) = self.get_width_height();
            offset *= -0.5;
            let x = width * offset;
            let y = height * offset;
            t.x += x;
            t.y += y;

            self.set_transform(t);
            t
        };

        self.run_callback(CoreMouseEvent::TransForm(t), p);
        let _ = self.render();
    }
}
