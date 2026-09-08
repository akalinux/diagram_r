use wasm_bindgen::prelude::*;
pub mod iters;
use crate::{
    DiagramOpt, Point,
    bsp::LookupPointResult,
    constants::{HALF, R_90, ZERO_POINT},
    link::iters::{ArcIter, FullBoxAccumulate, LineIter, LineIterSet},
    node::Node,
    square::Square,
    utils::{
        arc_contains_point, compute_arc_point, force_intersection, full_box_from, inside_box,
        inside_circle, normalize_rad,
    },
};
pub type AnimationLink = (Point, Point, f32);
#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Animation {
    Both,  // Animate in both directions
    ToSrc, // Animate towards the src node
    ToDst, // Animate towards the dst node
    None,  // Do not animate
}
#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    pub opt: usize,
    pub label: String,
    pub animation: Animation,
}

#[wasm_bindgen(inspectable)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinePoint {
    pub point: Point,
    pub mode: ArcType,
}
#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ArcType {
    Joint,
    Arc,
}

#[wasm_bindgen]
impl LinePoint {
    #[wasm_bindgen(constructor)]
    pub fn new(point: Point, mode: ArcType) -> Self {
        Self { point, mode }
    }
}
impl LinePoint {
    pub fn add_distance(&self, p: &Point) -> Self {
        let point = self.point.add_distance(p);
        Self {
            point,
            mode: self.mode,
        }
    }
    pub fn sub_distance(&self, p: &Point) -> Self {
        let point = self.point.sub_distance(p);
        Self {
            point,
            mode: self.mode,
        }
    }
    pub fn get_z_center(&self, b: &Point, c: &Point) -> Point {
        self.point.get_z_center(b, c)
    }
}
#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug, PartialEq)]
pub struct LinkSet {
    pub src: usize,
    pub dst: usize,

    #[wasm_bindgen(skip)]
    pub links: Vec<Link>,
    #[wasm_bindgen(skip)]
    pub bundles: Vec<Bundle>,
    pub point: Option<LinePoint>,
}

#[wasm_bindgen]
impl LinkSet {
    #[wasm_bindgen(constructor)]
    pub fn new(
        links: Vec<Link>,
        bundles: Vec<Bundle>,
        src: usize,
        dst: usize,
        point: Option<LinePoint>,
    ) -> Self {
        Self {
            src,
            dst,
            links,
            bundles,
            point,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineAnimation {
    None,
    Both(Box<[Point]>),
    Side(Box<[Point]>),
    BothArc(Box<[Point]>),
    SideArc(Box<[Point]>),
    JointBoth(Box<[Point]>),
    JointSide(Box<[Point]>),
}

fn move_points(list: &mut [Point], d: &Point) {
    for p in list {
        *p = p.add_distance(d);
    }
}
impl LineAnimation {
    pub fn move_distance(&mut self, d: &Point) {
        match self {
            LineAnimation::Side(list) => move_points(list, d),
            LineAnimation::Both(list) => move_points(list, d),
            LineAnimation::BothArc(list) => move_points(list, d),
            LineAnimation::SideArc(list) => move_points(list, d),
            LineAnimation::JointBoth(list) => move_points(list, d),
            LineAnimation::JointSide(list) => move_points(list, d),
            LineAnimation::None => (),
        }
    }
}

impl LinkSet {
    pub fn get_link_render_center(&self) {}
    pub fn compute_bunlde_points(&self, src: &Point, dst: &Point) -> Vec<Point> {
        let mut points = Vec::with_capacity(self.bundles.len());

        match &self.point {
            Some(arc) => match &arc.mode {
                ArcType::Arc => {
                    for bundle in &self.bundles {
                        points.push(compute_arc_point(bundle.pos, src, &arc.point, dst));
                    }
                }
                ArcType::Joint => {
                    for bundle in &self.bundles {
                        let (start, end) = match bundle.pos > HALF {
                            true => (&arc.point, dst),
                            false => (src, &arc.point),
                        };
                        let scale = bundle.pos * 2.0;
                        let d = start.get_move_distance(end);
                        points.push(start.add_distance(&d.scale(scale)));
                    }
                }
            },
            None => {
                let d = src.get_move_distance(dst);
                for bundle in &self.bundles {
                    points.push(src.add_distance(&d.scale(bundle.pos)));
                }
            }
        }
        points
    }

    pub fn arc_joint_animation(&self, r: f32, src: &Point, c: &Point, dst: &Point) -> [Point; 8] {
        let r2 = r * HALF;
        let d1 = src.get_distance_vec(c, r2, R_90);
        let d2 = c.get_distance_vec(&dst, r2, R_90);

        let vs = src.get_move_distance(c);
        let vc = c.get_move_distance(dst);
        let s1 = src.sub_distance(&d1);
        let e1 = dst.sub_distance(&d2);
        let c1 = {
            let c1 = c.sub_distance(&d1).add_distance(&vs);
            let c2 = c.sub_distance(&d2);
            force_intersection(&s1, &c1, &c2, &e1.add_distance(&vc))
        };
        let c2 = src.add_distance(&d1);
        let c3 = dst.add_distance(&d2);
        let p = {
            let s = c.add_distance(&d1).add_distance(&vs);
            let e = c.add_distance(&d2).sub_distance(&vc);
            force_intersection(&s, &c2, &c3, &e)
        };
        [
            s1, c1, c1, e1, // link1
            p, c2, c3, p, // Link 2
        ]
    }

    pub fn compute_animation(
        &self,
        link: &Link,
        src: &Point,
        dst: &Point,
        center: Option<(ArcType, &Point)>,
        r: f32,
    ) -> LineAnimation {
        match center {
            None => match link.animation {
                Animation::Both => {
                    let d = src.get_point(dst, r * HALF, R_90).get_move_distance(&src);
                    LineAnimation::Both(Box::new([
                        src.sub_distance(&d),
                        dst.sub_distance(&d),
                        dst.add_distance(&d),
                        src.add_distance(&d),
                    ]))
                }
                Animation::ToSrc => LineAnimation::Side(Box::new([*dst, *src])),
                Animation::ToDst => LineAnimation::Side(Box::new([*src, *dst])),
                _ => LineAnimation::None,
            },
            Some((t, c)) => match t {
                ArcType::Joint => match link.animation {
                    Animation::None => LineAnimation::None,
                    Animation::ToSrc => LineAnimation::JointSide(Box::new([*dst, *c, *src])),
                    Animation::ToDst => LineAnimation::JointSide(Box::new([*src, *c, *dst])),
                    Animation::Both => {
                        LineAnimation::JointBoth(Box::new(self.arc_joint_animation(r, src, c, dst)))
                    }
                },
                ArcType::Arc => match link.animation {
                    Animation::Both => {
                        //LineAnimation::BothArc(compute_arc_line_boundries(src, c, dst, r * HALF))
                        let [a, b, _, c, e, f, d, _] = self.arc_joint_animation(r, src, c, dst);
                        LineAnimation::BothArc(Box::new([a, b, c, d, e, f]))
                    }
                    Animation::ToSrc => LineAnimation::SideArc(Box::new([*dst, *c, *src])),
                    Animation::ToDst => LineAnimation::SideArc(Box::new([*src, *c, *dst])),
                    _ => LineAnimation::None,
                },
            },
        }
    }
    pub fn build_draw_data(&self, src: &Node, dst: &Node, opt: &DiagramOpt) -> DrawData {
        let src_p = src.layout.get_center();
        let dst_p = dst.layout.get_center();
        let mut accumulate = FullBoxAccumulate::new();
        let side = src.layout.smallest_side(&dst.layout) * opt.link_scale;

        let mut links = Vec::with_capacity(self.links.len());
        let (width, iter, mode, normalized_radians): (
            f32,
            Box<dyn LineIterSet>,
            ArcType,
            Box<[f32]>,
        ) = match &self.point {
            None => {
                let iter = LineIter::new(&src_p, &dst_p, side, self.links.len(), &mut accumulate);
                let width = iter.width;
                let rad = normalize_rad(src_p.get_radians(&dst_p));
                let i: Box<dyn LineIterSet> = Box::new(iter);

                (width, i, ArcType::Arc, Box::new([rad]))
            }
            Some(p) => {
                let iter = ArcIter::new(
                    &src_p,
                    &p.point,
                    &dst_p,
                    side,
                    self.links.len(),
                    &mut accumulate,
                );
                let width = iter.width;
                let set: Box<[f32]> = match p.mode {
                    ArcType::Arc => Box::new([iter.rad, if iter.swapped { -1.0 } else { 1.0 }]),
                    ArcType::Joint => {
                        let ra = normalize_rad(src_p.get_radians(&p.point));
                        let rb = normalize_rad(p.point.get_radians(&dst_p));
                        Box::new(if iter.swapped {
                            [rb, ra, -1.0]
                        } else {
                            [ra, rb, 1.0]
                        })
                    }
                };
                let i: Box<dyn LineIterSet> = Box::new(iter);

                (width, i, p.mode, set)
            }
        };
        let aw = width * HALF;
        let mut animated: usize = 0;

        for (link_id, (a, arc, b)) in iter.enumerate() {
            let link = &self.links[link_id];
            links.push(match arc {
                None => {
                    let animation = self.compute_animation(link, &a, &b, None, aw);
                    match &animation {
                        LineAnimation::None => (),
                        _ => animated += 1,
                    };

                    SubLink::Line([a, b], animation)
                }
                Some(c) => {
                    let animation = self.compute_animation(link, &a, &b, Some((mode, &c)), aw);
                    match &animation {
                        LineAnimation::None => (),
                        _ => animated += 1,
                    };
                    match mode {
                        ArcType::Arc => SubLink::Arc([a, c, b], animation),
                        ArcType::Joint => SubLink::Joint([a, c, b], animation),
                    }
                }
            });
        }

        let bundles = self.compute_bunlde_points(&src_p, &dst_p);
        let index = Square::from(accumulate.full_box_from());

        DrawData {
            line_width: width,
            bundle_side: side,
            bundles,
            links,
            index,
            normalized_radians,
            animated: animated != 0,
        }
    }
}

#[wasm_bindgen]
impl Link {
    #[wasm_bindgen(constructor)]
    pub fn new(opt: usize, label: String, animation: Animation) -> Self {
        Self {
            opt,
            label,
            animation,
        }
    }
}
#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug, PartialEq)]
pub struct Bundle {
    pub opt: usize,
    pub label: String,
    pub links: Vec<usize>,
    pub pos: f32,
}
#[wasm_bindgen]
impl Bundle {
    #[wasm_bindgen(constructor)]
    pub fn new(opt: usize, label: String, links: Vec<usize>, pos: f32) -> Self {
        Self {
            opt,
            label,
            links,
            pos,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SubLink {
    Line([Point; 2], LineAnimation),
    Joint([Point; 3], LineAnimation),
    Arc([Point; 3], LineAnimation),
}
impl SubLink {
    pub fn get_src_dst(&self) -> (Point, Point) {
        match self {
            Self::Arc([a, _, b], _) => (*a, *b),
            Self::Joint([a, _, b], _) => (*a, *b),
            Self::Line([a, b], _) => (*a, *b),
        }
    }
    pub fn sum_distance(&self) -> (usize, Point) {
        match &self {
            Self::Arc([a, b, c], _) => (3, a.add_distance(b).add_distance(c)),
            Self::Joint([a, b, c], _) => (3, a.add_distance(b).add_distance(c)),
            Self::Line([a, b], _) => (2, a.add_distance(b)),
        }
    }
    pub fn contains_point(&self, p: &Point, width: f32) -> bool {
        match self {
            Self::Joint([a, b, c], _) => {
                inside_circle(p, b, width)
                    || inside_box(&full_box_from(&a, &b, width), p)
                    || inside_box(&full_box_from(&b, &c, width), p)
            }
            Self::Arc([a, b, c], _) => arc_contains_point(width, p, a, b, c),
            Self::Line([a, b], _) => inside_box(&full_box_from(a, b, width), p),
        }
    }
    pub fn move_distance(&mut self, d: &Point) {
        match self {
            Self::Arc(a, b) => {
                move_points(a, d);
                b.move_distance(d);
            }
            Self::Joint(a, b) => {
                move_points(a, d);
                b.move_distance(d);
            }
            Self::Line(a, b) => {
                move_points(a, d);
                b.move_distance(d);
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DrawData {
    pub normalized_radians: Box<[f32]>,
    pub line_width: f32,
    pub bundle_side: f32,
    pub bundles: Vec<Point>,
    pub links: Vec<SubLink>,
    pub index: Square,
    pub animated: bool,
}

impl DrawData {
    pub fn get_src_dst(&self) -> (Point, Point) {
        let (mut src, mut dst) = self.links[0].get_src_dst();
        for i in 1..self.links.len() {
            let (a, b) = self.links[i].get_src_dst();
            src = src.add_distance(&a);
            dst = dst.add_distance(&b);
        }
        let scale = 1.0 / self.links.len() as f32;
        (src.scale(scale), dst.scale(scale))
    }
    pub fn bundle_draw_box(&self, i: usize) -> Square {
        let side = self.bundle_side;
        let offset = side * 0.5;
        let p = &self.bundles[i];
        Square::new(p.x - offset, p.y - offset, side, side)
    }

    pub fn move_distance(&mut self, distance: &Point) {
        self.index.move_distance(distance);
        for link in &mut self.links {
            link.move_distance(distance);
        }
        for bundle in &mut self.bundles {
            *bundle = bundle.add_distance(distance);
        }
    }
}

#[derive(Debug)]
pub struct LinkContainer {
    pub ls: LinkSet,
    pub draw_data: DrawData,
    pub id: usize,
}

impl LinkContainer {
    pub fn move_distance(&mut self, distance: &Point) {
        match &mut self.ls.point {
            Some(lp) => lp.point = lp.point.add_distance(distance),
            None => (),
        };
        self.draw_data.move_distance(distance);
    }
    pub fn move_arc(&mut self, distance: &Point, src: &Node, dst: &Node, opt: &DiagramOpt) {
        match &mut self.ls.point {
            Some(lp) => match lp.mode {
                ArcType::Arc => lp.point = lp.point.add_distance(&distance.scale(2.0)),
                ArcType::Joint => lp.point = lp.point.add_distance(distance),
            },
            None => (),
        }
        self.draw_data = self.ls.build_draw_data(src, dst, opt);
    }

    pub fn get_render_center(&self) -> Point {
        let (src, dst) = self.draw_data.get_src_dst();
        match &self.ls.point {
            Some(arc) => {
                match arc.mode {
                    ArcType::Arc => {
                        // this is not the arc center.. this is the apex center that the user sees
                        src.get_center(&dst).get_center(&arc.point)
                    }
                    ArcType::Joint => arc.point,
                }
            }
            None => src.get_center(&dst),
        }
    }
    pub fn contains_point(&self, p: &Point) -> LookupPointResult {
        let dd = &self.draw_data;
        let width = dd.line_width;
        let w = width * HALF;
        if let Some(arc) = &self.ls.point {
            let r = w + w * (dd.links.len() as f32) - w * HALF;
            match arc.mode {
                ArcType::Arc => {
                    let center = self.get_render_center();
                    if inside_circle(p, &center, r) {
                        return LookupPointResult::Arc(self.id);
                    }
                }
                ArcType::Joint => {
                    if inside_circle(p, &arc.point, r) {
                        return LookupPointResult::Arc(self.id);
                    }
                }
            }
        }
        // first check bundles
        for (i, _) in self.ls.bundles.iter().enumerate() {
            let square = dd.bundle_draw_box(i);
            if square.contains_point(p) {
                return LookupPointResult::Bundle((self.id, i));
            }
        }

        for (i, line) in dd.links.iter().enumerate() {
            if line.contains_point(p, w) {
                return LookupPointResult::Link((self.id, i));
            }
        }
        return LookupPointResult::NoMatch;
    }
    pub fn new(ls: LinkSet, src: &Node, dst: &Node, opt: &DiagramOpt, id: usize) -> Self {
        let dd = ls.build_draw_data(src, dst, opt);
        Self {
            ls,
            draw_data: dd,
            id,
        }
    }
    pub fn animated(&self) -> bool {
        self.draw_data.animated
    }
    pub fn get_src_dst(&self) -> (usize, usize) {
        (self.ls.src, self.ls.dst)
    }

    pub fn get_center(&self, check: &LookupPointResult) -> Point {
        let dd = &self.draw_data;
        match check {
            LookupPointResult::Arc(_) => unsafe { self.ls.point.unwrap_unchecked().point },
            LookupPointResult::Link(_) => {
                let mut start = ZERO_POINT;
                let mut count = 0;
                for sublink in &dd.links {
                    let (i, p) = sublink.sum_distance();
                    count += i;
                    start = start.add_distance(&p);
                }
                start.scale(1.0 / count as f32)
            }
            LookupPointResult::Bundle((i, _)) => dd.bundles[*i],
            _ => ZERO_POINT,
        }
    }
}
