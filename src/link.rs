use wasm_bindgen::prelude::*;
pub mod iters;
use crate::{
    DiagramOpt, Point,
    bsp::LookupPointResult,
    constants::{HALF, ZERO_POINT},
    link::iters::{ArcIter, FullBoxAccumulate, LineIter},
    node::Node,
    square::Square,
    utils::{full_box_from, inside_box, inside_circle},
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
    Both(f32, Point, Point, Point, Point),
    Side(f32, Point, Point),
    BothArc(f32, Point, Point, Point),
    SideArc(f32, Point, Point, Point),
}
pub type ComputedLink = (Point, Point, Option<LinePoint>);

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
        clink: &ComputedLink,
        width: f32,
        d: &Point,
    ) -> LineAnimation {
        match link.animation {
            Animation::Both => {
                let new_width = width * HALF;

                LineAnimation::Both(
                    new_width,
                    clink.0.sub_distance(&d),
                    clink.1.sub_distance(&d),
                    clink.1.add_distance(&d),
                    clink.0.add_distance(&d),
                )
            }
            Animation::ToSrc => LineAnimation::Side(width, clink.1, clink.0),
            Animation::ToDst => LineAnimation::Side(width, clink.0, clink.1),
            _ => LineAnimation::None,
        }
    }
    pub fn build_draw_data(&self, src: &Node, dst: &Node, opt: &DiagramOpt) -> DrawData {
        let src_p = src.layout.get_center();
        let dst_p = dst.layout.get_center();
        let mut accumulate = FullBoxAccumulate::new();
        let side = src.layout.smallest_side(&dst.layout) * opt.link_scale;

        let (links, width) = match self.point {
            None => {
                let iter = LineIter::new(&src_p, &dst_p, side, self.links.len());
                let animation_distance = iter.np.init.scale(0.25);
                let width = iter.width;
                let mut links = Vec::with_capacity(self.links.len());
                for (i, (a, b)) in iter.enumerate() {
                    let link = &self.links[i];
                    accumulate.step(&a);
                    accumulate.step(&b);
                    let animation = self.compute_animation(
                        link,
                        &(a, b, None),
                        width * HALF,
                        &animation_distance,
                    );
                    links.push((a, b, animation));
                }
                (links, width)
            }

            Some(lp) => {
                // unlke the above block, this generates 3 lines for every one link provided!!!
                let mut links = Vec::with_capacity(self.links.len() * 3);
                let iter = ArcIter::new(&src_p, &lp.point, &dst_p, side, self.links.len());
                let width = iter.width;
                let animation_distance_start = iter.a.init.scale(0.25);
                let animation_distance_end = iter.b.init.scale(0.25);
                for (i, (a, b, c)) in iter.enumerate() {
                    let link = &self.links[i];
                    accumulate.step(&a);
                    accumulate.step(&b);
                    accumulate.step(&c);
                    for (a, b, animation_distance) in [
                        (a, b, animation_distance_start),
                        (b, c, animation_distance_end),
                    ] {
                        let animation = self.compute_animation(
                            link,
                            &(a, b, None),
                            width * HALF,
                            &animation_distance,
                        );
                        links.push((a, b, animation));
                    }
                }

                (links, width)
            }
        };
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

pub type SubLink = (Point, Point, LineAnimation);

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
            link.0 = link.0.add_distance(distance);
            link.1 = link.1.add_distance(distance);

            match &mut link.2 {
                LineAnimation::None => (),
                LineAnimation::Both(_, a, b, c, d) => {
                    for p in [a, b, c, d] {
                        *p = p.add_distance(distance);
                    }
                }
                LineAnimation::Side(_, a, b) => {
                    *a = a.add_distance(distance);
                    *b = b.add_distance(distance);
                }
                _ => (),
            }
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
        for (i, _) in self.ls.links.iter().enumerate() {
            let (src, dst, _) = &dd.links[i];
            match &self.ls.point {
                Some(arc) => match &arc.mode {
                    ArcType::Arc => todo!("FIXME!"),
                    ArcType::Joint => {
                        let dd = &self.draw_data;
                        let width = dd.line_width * HALF;
                        if inside_circle(p, &arc.point, width) {
                            return LookupPointResult::Arc(self.id);
                        }
                        for i in 0..self.ls.links.len() {
                            for o in 0..2 {
                                let id = i * 2 + o;
                                let link = &dd.links[id];
                                if o == 0 {
                                    if inside_circle(p, &link.1, width) {
                                        return LookupPointResult::Link((self.id, i));
                                    }
                                }
                                let (pb, _) = full_box_from(&link.0, &link.1, width);
                                if inside_box(&pb, p) {
                                    return LookupPointResult::Link((self.id, i));
                                }
                            }
                        }
                    }
                },
                None => {
                    let (pb, _) = full_box_from(src, dst, dd.line_width * HALF);
                    if inside_box(&pb, p) {
                        return LookupPointResult::Link((self.id, i));
                    }
                }
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
                for (a, b, _) in &dd.links {
                    start = start.add_distance(a).add_distance(b);
                }
                start.scale(1.0 / dd.links.len() as f32)
            }
            LookupPointResult::Bundle((i, _)) => dd.bundles[*i],
            _ => ZERO_POINT,
        }
    }
}
