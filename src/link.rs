use wasm_bindgen::prelude::*;
pub mod iters;
use crate::{
    DiagramOpt, Point,
    bsp::LookupPointResult,
    constants::{HALF, ZERO_POINT},
    node::Node,
    square::Square,
    utils::{
        angle_needs_normalization, compute_line_box, full_box_from, get_angle, get_xy, inside_box,
        inside_circle,
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

pub fn compute_animation(
    link: &Link,
    clink: &ComputedLink,
    width: f32,
    d: &Point,
) -> Option<LineAnimation> {
    match link.animation {
        Animation::Both => {
            let new_width = width * HALF;

            match &clink.2 {
                Some(arc) => match &arc.mode {
                    ArcType::Arc => None,
                    ArcType::Joint => {
                        let mut links = Vec::with_capacity(4);

                        for (a, b) in [(&clink.0, &arc.point), (&clink.1, &arc.point)] {
                            let angle = get_angle(a.x, a.y, b.x, b.y);
                            let s = get_xy(a.x, a.y, new_width, angle + 90.0);
                            let d = s.get_move_distance(a);
                            links.push((a.add_distance(&d), b.add_distance(&d), None, new_width));
                            links.push((b.sub_distance(&d), a.sub_distance(&d), None, new_width));
                        }

                        Some(links)
                    }
                },
                None => Some(vec![
                    (
                        clink.0.sub_distance(&d),
                        clink.1.sub_distance(&d),
                        None,
                        new_width,
                    ),
                    (
                        clink.1.add_distance(&d),
                        clink.0.add_distance(&d),
                        None,
                        new_width,
                    ),
                ]),
            }
        }
        Animation::ToSrc => Some(vec![(clink.1, clink.0, clink.2, width)]),
        Animation::ToDst => Some(vec![(clink.0, clink.1, clink.2, width)]),
        _ => None,
    }
}

pub type LineAnimation = Vec<(Point, Point, Option<LinePoint>, f32)>;
pub type ComputedLink = (Point, Point, Option<LinePoint>);

pub fn get_line_width(total_links: usize, full_width: f32) -> (f32, f32, f32) {
    let incremental_scale = 1.0 / total_links as f32;
    let (virtual_count, inital_scale) = match total_links {
        1 => (2.0, 0.5),
        _ => (total_links as f32 * 2.0 - 1.0, incremental_scale * 0.5),
    };
    let link_width = full_width / virtual_count;
    (link_width, inital_scale, incremental_scale)
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

    pub fn build_draw_data(&self, src: &Node, dst: &Node, opt: &DiagramOpt) -> DrawData {
        let src_p = src.layout.get_center();
        let dst_p = dst.layout.get_center();
        let smallest_side = src.layout.smallest_side(&dst.layout);
        let r = smallest_side * 0.5;
        let ((nw, ne, sw, se), (d, _, angle)) = full_box_from(&src_p, &dst_p, r);

        let idx = match &self.point {
            None => Square::from(compute_line_box(&ne, &[&nw, &se, &sw])),
            Some(arc) => match &arc.mode {
                ArcType::Joint => Square::from(compute_line_box(&ne, &[&nw, &se, &sw, &arc.point])),
                ArcType::Arc => todo!("will get there"),
            },
        };

        let (width, inital_scale, scale) = get_line_width(self.links.len(), smallest_side);

        let mut links = Vec::with_capacity(self.links.len());

        let (distance, left, right) = match angle_needs_normalization(angle) {
            false => (d.scale(-2.0), sw, se),
            true => (d.scale(2.0), nw, ne),
        };
        let init = distance.scale(inital_scale);
        let chunk = distance.scale(scale);
        let start = left.add_distance(&init);
        let end = right.add_distance(&init);
        let sublink_width = width * HALF;
        let animation_distance = init.scale(0.25);

        for (i, link) in self.links.iter().enumerate() {
            links.push(self.build_sublink(
                link,
                &start,
                &end,
                &chunk,
                i,
                sublink_width,
                &animation_distance,
            ));
        }
        let bundles = self.compute_bunlde_points(&src_p, &dst_p);

        DrawData {
            line_width: width,
            bundle_side: smallest_side * opt.link_scale,
            bundles,
            links,
            index: idx,
        }
    }
    fn build_sublink(
        &self,
        link: &Link,
        nw: &Point,
        ne: &Point,
        chunk: &Point,
        i: usize,
        width: f32,
        animation_distance: &Point,
    ) -> (Point, Point, Option<LinePoint>, Option<LineAnimation>) {
        let ix = i as f32;
        let d = chunk.scale(ix);
        let start = nw.add_distance(&d);
        let end = ne.add_distance(&d);
        let arc = match &self.point {
            Some(lp) => {
                let center = lp.add_distance(&d);
                Some(center)
            }
            None => None,
        };

        let animation = compute_animation(link, &(start, end, arc), width, animation_distance);
        (start, end, arc, animation)
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

pub type SubLink = (Point, Point, Option<LinePoint>, Option<LineAnimation>);
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
                Some(arc) => arc.point = arc.point.add_distance(distance),
                None => (),
            };
            if let Some(list) = &mut link.3 {
                for i in 0..list.len() {
                    let (src, dst, ap, _) = &mut list[i];

                    *src = src.add_distance(distance);
                    *dst = dst.add_distance(distance);
                    match ap {
                        Some(lp) => lp.point = lp.point.add_distance(distance),
                        _ => (),
                    }
                }
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
            let (src, dst, ap, _) = &dd.links[i];
            match ap {
                Some(arc) => match &arc.mode {
                    ArcType::Arc => todo!("FIXME!"),
                    ArcType::Joint => {
                        let center = &arc.point;
                        let width = dd.line_width * HALF;
                        if inside_circle(p, &arc.point, width) {
                            return LookupPointResult::Arc(self.id);
                        }

                        let (mut pb, _) = full_box_from(src, center, width);
                        if inside_box(&pb, p) {
                            return LookupPointResult::Link((self.id, i));
                        }
                        (pb, _) = full_box_from(center, dst, width);
                        if inside_box(&pb, p) {
                            return LookupPointResult::Link((self.id, i));
                        }
                    }
                },
                None => {
                    let (pb, _) = full_box_from(src, dst, dd.line_width * 0.5);
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
            LookupPointResult::NoMatch => {
                let mut x = 0.0;
                let mut y = 0.0;
                // TODO
                for (a, b, _, _) in &dd.links {
                    x += a.x + b.x;
                    y += a.y + b.y;
                }
                let count = (self.ls.links.len() * 2) as f32;
                Point {
                    x: x / count,
                    y: y / count,
                }
            }
            LookupPointResult::Link((i, _)) => {
                let (a, b, x, _) = &dd.links[*i];
                match x {
                    Some(c) => c.get_z_center(a, b),
                    None => a.get_center(&b),
                }
            }
            LookupPointResult::Bundle((i, _)) => dd.bundles[*i],
            _ => ZERO_POINT,
        }
    }
}
