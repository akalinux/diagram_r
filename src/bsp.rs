use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};

use crate::{
    Point,
    diagram::DiagramCore,
    link::{Bundle, Link},
};

pub type IndexXY = (RangeInclusive<i64>, RangeInclusive<i64>);

#[derive(PartialEq, Debug)]
pub enum LookupPointResult {
    Bundle((Bundle, usize)),
    Link((Link, usize)),
    Node(u32),
    Box(u32),
    Screen,
    NoMatch,
}

enum IdxBoxIterSection {
    Old,
    New,
    Done,
}

#[derive(Clone, Copy, Hash, Debug)]
pub enum ScreenSlot {
    Node(u32),
    Box(u32),
    Link(u64),
}

pub struct IdxBoxIter {
    old: IndexXY,
    new: IndexXY,
    step: i64,
    next: Option<(i64, i64, IdxBoxIterSection)>,
}
pub struct ScreenBoundY {
    pub nodes: HashSet<u32>,
    pub boxes: HashSet<u32>,
    pub links: HashSet<u64>,
}

#[derive(Hash, PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
pub struct XY {
    x: i64,
    y: i64,
}
pub struct ScreenIndex {
    pub step: i64,
    x: HashMap<XY, ScreenBoundY>,
}

impl ScreenIndex {
    pub fn clear(&mut self) {
        self.x.clear();
    }
    pub fn new(step: i64) -> Self {
        Self {
            step,
            x: HashMap::new(),
        }
    }
    fn step(&self) -> usize {
        self.step as usize
    }
    pub fn update(&mut self, dst: &ScreenSlot, old: IndexXY, new: IndexXY) {
        for (x, y, action) in IdxBoxIter::new(old, new, self.step) {
            let points = (x..=x, y..=y);
            self.manage(dst, points, action);
        }
    }
    pub fn contains_point(&self, p: &Point, d: &DiagramCore) -> LookupPointResult {
        let (x, y) = p.idx(self.step);
        let z = XY { x, y };
        if let Some(screen) = self.x.get(&z) {
            for id in screen.nodes.iter() {
                let node = unsafe { d.nodes.get(id).unwrap_unchecked() }.get();
                if node.layout.contains_point(p) {
                    return LookupPointResult::Node(*id);
                }
            }
            for id in screen.links.iter() {
                let lc = unsafe { d.links.get(id).unwrap_unchecked() };
                match lc.contains_point(p) {
                    LookupPointResult::Bundle(data) => return LookupPointResult::Bundle(data),
                    LookupPointResult::Link(data) => return LookupPointResult::Link(data),
                    _ => (),
                }
            }
            for id in screen.boxes.iter() {
                let node = unsafe { d.nodes.get(id).unwrap_unchecked() }.get();
                if node.layout.contains_point(p) {
                    return LookupPointResult::Box(*id);
                }
            }
        }
        LookupPointResult::Screen
    }
    pub fn manage(&mut self, dst: &ScreenSlot, points: IndexXY, action: IdxBoxAction) {
        let (px, py) = points;
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
                    IdxBoxAction::Add => tx.add(dst),
                    IdxBoxAction::Remove => tx.remove(dst),
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
            nodes: HashSet::new(),
            boxes: HashSet::new(),
            links: HashSet::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        return self.nodes.is_empty() && self.links.is_empty() && self.boxes.is_empty();
    }

    pub fn add(&mut self, t: &ScreenSlot) {
        match t {
            ScreenSlot::Link(l) => self.links.insert(*l),
            ScreenSlot::Node(n) => self.nodes.insert(*n),
            ScreenSlot::Box(n) => self.boxes.insert(*n),
        };
    }
    pub fn remove(&mut self, t: &ScreenSlot) {
        match t {
            ScreenSlot::Link(l) => self.links.remove(l),
            ScreenSlot::Node(n) => self.nodes.remove(n),
            ScreenSlot::Box(n) => self.boxes.remove(n),
        };
    }
}
impl IdxBoxIter {
    pub fn new(old: IndexXY, new: IndexXY, step: i64) -> Self {
        if old.0 == new.0 && old.1 == new.1 {
            return Self {
                old,
                new,
                step,
                next: None,
            };
        }
        let x = *old.0.start();
        let y = *old.1.start();
        Self {
            old,
            new,
            step,
            next: Some((x, y, IdxBoxIterSection::Old)),
        }
    }
    fn h_next(x: &mut i64, y: &mut i64, s: &mut IdxBoxIterSection, n: &IndexXY) {
        match s {
            IdxBoxIterSection::Old => {
                *x = *n.0.start();
                *y = *n.1.start();
                *s = IdxBoxIterSection::New;
            }
            _ => *s = IdxBoxIterSection::Done,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IdxBoxAction {
    Add,
    Remove,
}
impl Iterator for IdxBoxIter {
    type Item = (i64, i64, IdxBoxAction);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.next {
                Some((cx, cy, s)) => {
                    let a;
                    let b;
                    let t;
                    match s {
                        IdxBoxIterSection::Old => {
                            a = &self.old;
                            b = &self.new;
                            t = IdxBoxAction::Remove;
                        }
                        IdxBoxIterSection::New => {
                            a = &self.new;
                            b = &self.old;
                            t = IdxBoxAction::Add;
                        }
                        IdxBoxIterSection::Done => return None,
                    }
                    let x = *cx;
                    let y = *cy;
                    let (cmp_x, cmp_y) = b;
                    if cmp_x.contains(&x) && cmp_y.contains(&y) {
                        *cx = cmp_x.end() + self.step;
                        continue;
                    }

                    if x > *a.0.end() {
                        *cx = *a.0.start();
                        *cy += self.step;
                        continue;
                    } else if y > *a.1.end() {
                        Self::h_next(cx, cy, s, b);
                        continue;
                    }
                    *cx += self.step;
                    return Some((x, y, t));
                }
                None => return None,
            }
        }
    }
}
