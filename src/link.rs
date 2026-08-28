use wasm_bindgen::prelude::*;
pub mod iters;
use crate::{
    DiagramOpt, Point,
    bsp::LookupPointResult,
    constants::{HALF, R_90, ZERO_POINT},
    link::iters::{ArcIter, FullBoxAccumulate, LineIter, LineIterSet},
    node::Node,
    square::Square,
    utils::{force_intersection, full_box_from, inside_box, inside_circle},
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineAnimation {
    None,
    Both([Point; 4]),
    Side([Point; 2]),
    BothArc([Point; 8]),
    SideArc([Point; 3]),
    JointBoth([Point; 8]),
    JointSide([Point; 3]),
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
    pub fn compute_bunlde_points(&self, src: &Point, dst: &Point) -> Vec<Point> {
        let mut points = Vec::with_capacity(self.bundles.len());

        match &self.point {
            Some(arc) => match &arc.mode {
                ArcType::Arc => todo!("Not there yet!"),
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

    pub fn compute_animation(
        &self,
        link: &Link,
        clink: (&Point, &Point, Option<(ArcType, &Point)>),
        r: f32,
    ) -> LineAnimation {
        match clink.2 {
            None => match link.animation {
                Animation::Both => {
                    let d = clink
                        .0
                        .get_point(&clink.1, r * HALF, R_90)
                        .get_move_distance(&clink.0);
                    LineAnimation::Both([
                        clink.0.sub_distance(&d),
                        clink.1.sub_distance(&d),
                        clink.1.add_distance(&d),
                        clink.0.add_distance(&d),
                    ])
                }
                Animation::ToSrc => LineAnimation::Side([*clink.1, *clink.0]),
                Animation::ToDst => LineAnimation::Side([*clink.0, *clink.1]),
                _ => LineAnimation::None,
            },
            Some((t, c)) => match t {
                ArcType::Joint => {
                    match link.animation {
                        Animation::None => LineAnimation::None,
                        Animation::ToDst => LineAnimation::JointSide([*clink.0, *c, *clink.1]),
                        Animation::ToSrc => LineAnimation::JointSide([*clink.1, *c, *clink.0]),
                        Animation::Both => {
                            let r2 = r * HALF;

                            let (src, dst, _) = clink;
                            let d1 = src.get_distance_vec(c, r2, R_90);
                            let d2 = c.get_distance_vec(&dst, r2, R_90);

                            LineAnimation::JointBoth([
                                // link1
                                src.sub_distance(&d1),
                                c.sub_distance(&d1),
                                c.sub_distance(&d2),
                                dst.sub_distance(&d2),
                                // Link 2
                                c.add_distance(&d1),
                                src.add_distance(&d1),
                                dst.add_distance(&d2),
                                c.add_distance(&d2),
                            ])
                        }
                    }
                }
                ArcType::Arc => LineAnimation::None, // TODO
            },
        }
    }
    pub fn build_draw_data(&self, src: &Node, dst: &Node, opt: &DiagramOpt) -> DrawData {
        let src_p = src.layout.get_center();
        let dst_p = dst.layout.get_center();
        let mut accumulate = FullBoxAccumulate::new();
        let side = src.layout.smallest_side(&dst.layout) * opt.link_scale;

        let mut links = Vec::with_capacity(self.links.len());
        let (width, iter, mode) = match &self.point {
            None => {
                let iter = LineIter::new(&src_p, &dst_p, side, self.links.len());
                let width = iter.width;
                let i: Box<dyn LineIterSet> = Box::new(iter);
                (width, i, ArcType::Arc)
            }
            Some(p) => {
                let iter = ArcIter::new(&src_p, &p.point, &dst_p, side, self.links.len());
                let width = iter.width;
                let i: Box<dyn LineIterSet> = Box::new(iter);

                (width, i, p.mode)
            }
        };
        let aw = width * HALF;
        for (link_id, (a, arc, b)) in iter.enumerate() {
            accumulate.step(&a);
            accumulate.step(&b);
            let link = &self.links[link_id];
            links.push(match arc {
                None => {
                    let animation = self.compute_animation(link, (&a, &b, None), aw);
                    SubLink::Line([a, b], animation)
                }
                Some(c) => {
                    let animation = self.compute_animation(link, (&a, &b, Some((mode, &c))), aw);
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
                    || inside_box(&full_box_from(&a, &b, width).0, p)
                    || inside_box(&full_box_from(&b, &c, width).0, p)
            }
            Self::Arc([_, _, _], _) => false, // TODO!
            Self::Line([a, b], _) => inside_box(&full_box_from(a, b, width).0, p),
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
    pub line_width: f32,
    pub bundle_side: f32,
    pub bundles: Vec<Point>,
    pub links: Vec<SubLink>,
    pub index: Square,
}

impl DrawData {
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
            Some(lp) => lp.point = lp.point.add_distance(distance),
            None => (),
        }
        self.draw_data = self.ls.build_draw_data(src, dst, opt);
    }
    pub fn contains_point(&self, p: &Point) -> LookupPointResult {
        let dd = &self.draw_data;
        // first check bundles
        for (i, _) in self.ls.bundles.iter().enumerate() {
            let square = dd.bundle_draw_box(i);
            if square.contains_point(p) {
                return LookupPointResult::Bundle((self.id, i));
            }
        }
        let width = dd.line_width;
        let w = width * HALF;
        if let Some(arc) = &self.ls.point {
            let r = w + w * (dd.links.len() as f32) - w * HALF;
            if inside_circle(p, &arc.point, r) {
                return LookupPointResult::Arc(self.id);
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
