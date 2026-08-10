use rustc_hash::FxHashMap;
use std::{collections::BTreeSet, ops::RangeInclusive};
pub mod iter;
use crate::{
    Point,
    bsp::iter::{IdxBoxAction, IdxBoxIter},
    diagram::{DiagramCore, NodeCanvasTarget},
    link::{Bundle, Link},
};

pub type IndexXY = (RangeInclusive<i64>, RangeInclusive<i64>, f64);
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum Slot {
    Node,
    Link,
    Box,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum Id {
    Link(usize),
    Node(usize),
}

impl Id {
    fn get_link(&self) -> Option<usize> {
        match self {
            Id::Link(id) => Some(*id),
            _ => None,
        }
    }
    fn get_node(&self) -> Option<usize> {
        match self {
            Id::Node(id) => Some(*id),
            _ => None,
        }
    }
    fn link(&self) -> usize {
        unsafe { self.get_link().unwrap_unchecked() }
    }
    fn node(&self) -> usize {
        unsafe { self.get_node().unwrap_unchecked() }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct XYSet {
    slot: Slot,
    size: f64,
    id: Id,
}
impl PartialEq for XYSet {
    fn eq(&self, other: &Self) -> bool {
        self.slot == other.slot && self.size == other.size && self.id == other.id
    }
}
impl Eq for XYSet {}
impl PartialOrd for XYSet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.slot.partial_cmp(&other.slot) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.size.partial_cmp(&other.size) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for XYSet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        unsafe { self.partial_cmp(other).unwrap_unchecked() }
    }
}

#[derive(PartialEq, Debug)]
pub enum LookupPointResult {
    Bundle((Bundle, usize, usize)),
    Link((Link, usize, usize)),
    Node(usize),
    Box(usize),
    Screen,
    NoMatch,
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScreenSlot {
    Node(usize),
    Box(usize),
    Link(usize),
}

impl ScreenSlot {
    pub fn get_link_id(&self) -> Option<&usize> {
        if let ScreenSlot::Link(id) = self {
            return Some(id);
        }
        None
    }
    pub fn get_box_id(&self) -> Option<&usize> {
        if let ScreenSlot::Box(id) = self {
            return Some(id);
        }
        None
    }
    pub fn get_node_id(&self) -> Option<&usize> {
        if let ScreenSlot::Node(id) = self {
            return Some(id);
        }
        None
    }
}

pub struct ScreenBoundY {
    pub nodes: BTreeSet<XYSet>,
}

#[derive(Hash, PartialEq, PartialOrd, Eq, Ord, Copy, Clone, Debug)]
pub struct XY {
    x: i64,
    y: i64,
}
pub struct ScreenIndex {
    pub step: i64,
    x: FxHashMap<XY, ScreenBoundY>,
}

impl ScreenIndex {
    pub fn clear(&mut self) {
        self.x.clear();
    }
    pub fn new(step: i64) -> Self {
        Self {
            step,
            x: FxHashMap::default(),
        }
    }
    fn step(&self) -> usize {
        self.step as usize
    }
    pub fn update(&mut self, dst: &ScreenSlot, old: IndexXY, new: IndexXY) {
        for (x, y, action, area) in IdxBoxIter::new(old, new, self.step) {
            let points = (x..=x, y..=y, area);
            self.manage(dst, points, action);
        }
    }
    pub fn contains_point(&self, p: &Point, d: &DiagramCore) -> LookupPointResult {
        let (x, y) = p.idx(self.step);
        let z = XY { x, y };
        if let Some(screen) = self.x.get(&z) {
            for set in screen.nodes.iter() {
                match set.slot {
                    Slot::Link => {
                        let lid = set.id.link();
                        match unsafe { d.links.get(lid as usize).unwrap_unchecked() }
                            .contains_point(p)
                        {
                            LookupPointResult::Link(res) => return LookupPointResult::Link(res),
                            LookupPointResult::Bundle(res) => {
                                return LookupPointResult::Bundle(res);
                            }
                            _ => (),
                        }
                    }
                    Slot::Box | Slot::Node => {
                        let id = &set.id.node();
                        match &d.nodes[*id as usize] {
                            NodeCanvasTarget::Box((node, _)) => {
                                if node.layout.contains_point(p) {
                                    return LookupPointResult::Box(*id);
                                }
                            }
                            NodeCanvasTarget::Node((node, _)) => {
                                if node.layout.contains_point(p) {
                                    return LookupPointResult::Node(*id);
                                }
                            }
                        }
                    }
                }
            }
        }
        LookupPointResult::Screen
    }
    pub fn manage(&mut self, dst: &ScreenSlot, points: IndexXY, action: IdxBoxAction) {
        let (px, py, area) = points;
        let step = self.step();
        for x in px.step_by(step) {
            for y in py.clone().step_by(step) {
                let tx;
                let y = XY { x, y };
                if let Some(t) = self.x.get_mut(&y) {
                    tx = t
                } else {
                    match action {
                        IdxBoxAction::Remove => continue,
                        _ => (),
                    }
                    self.x.insert(y, ScreenBoundY::new());
                    tx = unsafe { self.x.get_mut(&y).unwrap_unchecked() }
                }
                match action {
                    IdxBoxAction::Add => tx.add(dst, area),
                    IdxBoxAction::Remove => tx.remove(dst, area),
                }
                if tx.is_empty() {
                    self.x.remove(&y);
                }
            }
        }
    }
}

impl ScreenBoundY {
    pub fn new() -> Self {
        Self {
            nodes: BTreeSet::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        return self.nodes.is_empty();
    }

    pub fn add(&mut self, t: &ScreenSlot, area: f64) {
        match t {
            ScreenSlot::Link(l) => self.nodes.insert(XYSet {
                slot: Slot::Link,
                size: area,
                id: Id::Link(*l),
            }),
            ScreenSlot::Node(n) => self.nodes.insert(XYSet {
                slot: Slot::Node,
                size: area,
                id: Id::Node(*n),
            }),
            ScreenSlot::Box(n) => self.nodes.insert(XYSet {
                slot: Slot::Box,
                size: area,
                id: Id::Node(*n),
            }),
        };
    }
    pub fn remove(&mut self, t: &ScreenSlot, area: f64) {
        match t {
            ScreenSlot::Link(l) => self.nodes.remove(&XYSet {
                slot: Slot::Link,
                size: area,
                id: Id::Link(*l),
            }),
            ScreenSlot::Node(n) => self.nodes.remove(&XYSet {
                slot: Slot::Node,
                size: area,
                id: Id::Node(*n),
            }),
            ScreenSlot::Box(n) => self.nodes.remove(&XYSet {
                slot: Slot::Box,
                size: area,
                id: Id::Node(*n),
            }),
        };
    }
}
