use std::mem;

use wasm_bindgen::prelude::*;

use crate::{
    Point,
    diagram::DiagramOpt,
    node::Node,
    square::Square,
    utils::{
        compute_line_box, create_container_id, full_box_from, get_angle, get_distance, get_xy,
        inside_box,
    },
};
pub type AnimationLink = (Point, Point, f64);
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
    pub src: u32,
    pub dst: u32,
    pub opt: u32,
    pub label: String,
    pub animation: Animation,
}
#[wasm_bindgen]
impl Link {
    #[wasm_bindgen(constructor)]
    pub fn new(src: u32, dst: u32, opt: u32, label: String, animation: Animation) -> Self {
        Self {
            src,
            dst,
            opt,
            label,
            animation,
        }
    }
    pub fn link_id(&self) -> u64 {
        create_container_id(self.src, self.dst)
    }
}
#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug, PartialEq)]
pub struct Bundle {
    pub src: u32,
    pub dst: u32,
    pub opt: u32,
    pub label: String,
    pub links: Vec<usize>,
}
#[wasm_bindgen]
impl Bundle {
    #[wasm_bindgen(constructor)]
    pub fn new(src: u32, dst: u32, opt: u32, label: String, links: Vec<usize>) -> Self {
        Self {
            src,
            dst,
            opt,
            label,
            links,
        }
    }
    pub fn link_id(&self) -> u64 {
        create_container_id(self.src, self.dst)
    }
}

#[derive(Debug, PartialEq)]
pub struct DrawData {
    pub line_width: f64,

    pub bundle_side: f64,
    pub bundles: Vec<Point>,
    pub links: Vec<(Point, Point)>,
    pub animations: Vec<AnimationLink>,
    pub index: Square,
}
impl DrawData {
    pub fn bundle_draw_box(&self, i: usize) -> Square {
        let side = self.bundle_side;
        let offset = side * 0.5;
        let p = &self.bundles[i];
        Square::new(p.x - offset, p.y - offset, side, side)
    }
}
pub struct LinkContainer {
    pub links: Vec<Link>,
    pub bundles: Vec<Bundle>,
    pub draw_data: Option<DrawData>,
    pub id: u64,
}

#[derive(PartialEq, Debug)]
pub enum PointInLink {
    Bundle((Bundle, usize)),
    Link((Link, usize)),
    NoMatch,
}

impl LinkContainer {
    pub fn contains_point(&self, p: &Point) -> PointInLink {
        match &self.draw_data {
            Some(dd) => {
                // first check bundles
                for (i, b) in self.bundles.iter().enumerate() {
                    let square = dd.bundle_draw_box(i);
                    if square.contains_point(p) {
                        return PointInLink::Bundle((b.clone(), i));
                    }
                }
                for (i, l) in self.links.iter().enumerate() {
                    let lp = &dd.links[i];
                    let (pb, _) = full_box_from(&lp.0, &lp.1, dd.line_width);
                    if inside_box(&pb, p) {
                        return PointInLink::Link((l.clone(), i));
                    }
                }
                return PointInLink::NoMatch;
            }
            None => PointInLink::NoMatch,
        }
    }
    pub fn new(id: u64) -> Self {
        Self {
            links: Vec::new(),
            bundles: Vec::new(),
            draw_data: None,
            id,
        }
    }
    pub fn get_src_dst(&self) -> (u32, u32) {
        return unsafe { mem::transmute::<u64, (u32, u32)>(self.id) };
    }
    pub fn add_link(&mut self, link: Link) {
        self.links.push(link);
    }

    pub fn add_bundle(&mut self, bundle: Bundle) -> Result<(), JsValue> {
        for id in &bundle.links {
            if !(self.links.len() > *id) {
                return Err(JsValue::from_str("Bundle Points to an invalid link"));
            }
        }
        self.bundles.push(bundle);
        Ok(())
    }

    pub fn update(&mut self, src: &Node, dst: &Node, opt: &DiagramOpt) {
        let src_p = src.layout.get_center();
        let dst_p = dst.layout.get_center();
        let smallest_side = src.layout.smallest_side(&dst.layout);
        let r = smallest_side * 0.5;
        let ((nw, ne, sw, se), (_, north, south)) = full_box_from(&src_p, &dst_p, r);
        let idx = Square::from(compute_line_box(&ne, [&nw, &se, &sw]));
        let (width, step, init_step) =
            self.compute_line_width(opt.link_scale, smallest_side, self.links.len());

        let mut animations = Vec::new(); // no way to know how big this will be :/
        let mut bundles = Vec::with_capacity(self.bundles.len());
        let mut links = Vec::with_capacity(self.links.len());

        for (i, link) in self.links.iter().enumerate() {
            let inc_by = init_step + step * (i as f64);
            let start = get_xy(nw.x, nw.y, inc_by, south);
            let end = get_xy(ne.x, ne.y, inc_by, south);
            let clink = (start, end);
            self.compute_animation(link, &clink, &mut animations, width, north, south);

            links.push(clink);
        }
        self.compute_bunlde_points(&src_p, &dst_p, self.bundles.len(), &mut bundles);

        self.draw_data = Some(DrawData {
            line_width: width,
            bundle_side: smallest_side * opt.link_scale,
            bundles,
            links,
            animations,
            index: idx,
        })
    }

    pub fn get_center(&self, check: &PointInLink) -> Point {
        let dd = unsafe { self.draw_data.as_ref().unwrap_unchecked() };
        match check {
            PointInLink::NoMatch => {
                let mut x = 0.0;
                let mut y = 0.0;
                for (a, b) in &dd.links {
                    x += a.x + b.x;
                    y += a.y + b.y;
                }
                let count = (self.links.len() * 2) as f64;
                Point {
                    x: x / count,
                    y: y / count,
                }
            }
            PointInLink::Link((_, i)) => {
                let (a, b) = dd.links[*i];
                Point {
                    x: (a.x + b.x) / 2.0,
                    y: (a.y + b.y) / 2.0,
                }
            }
            PointInLink::Bundle((_, i)) => dd.bundles[*i],
        }
    }

    pub fn compute_bunlde_points(
        &self,
        src: &Point,
        dst: &Point,
        bundles: usize,
        points: &mut Vec<Point>,
    ) {
        let distance = get_distance(src.x, src.y, dst.x, dst.y);
        let scale = (bundles * 2) as f64;
        let size = distance / scale;

        let angle = get_angle(src.x, src.y, dst.x, dst.y) + 180.0;
        for i in (1..bundles * 2).step_by(2) {
            let r = size * i as f64;
            points.push(get_xy(src.x, src.y, r, angle));
        }
    }
    pub fn compute_animation(
        &self,
        link: &Link,
        clink: &(Point, Point),
        animations: &mut Vec<AnimationLink>,
        width: f64,
        angle_north: f64,
        angle_south: f64,
    ) {
        match link.animation {
            Animation::Both => {
                let (aw, _, init_step) = self.compute_line_width(1.0, width, 2);

                animations.push((
                    get_xy(clink.0.x, clink.0.y, init_step, angle_north),
                    get_xy(clink.1.x, clink.1.y, init_step, angle_north),
                    aw,
                ));
                animations.push((
                    get_xy(clink.0.x, clink.0.y, init_step, angle_south),
                    get_xy(clink.1.x, clink.1.y, init_step, angle_south),
                    aw,
                ));
            }
            Animation::ToSrc => {
                let (aw, _, _) = self.compute_line_width(1.0, width, 1);
                animations.push((clink.1, clink.0, aw));
            }
            Animation::ToDst => {
                let (aw, _, _) = self.compute_line_width(1.0, width, 1);

                animations.push((clink.0, clink.1, aw));
            }
            _ => (),
        }
    }

    pub fn compute_line_width(&self, link_scale: f64, r: f64, nodes: usize) -> (f64, f64, f64) {
        //let lc = self.compute_node_scale(nodes) as f64;
        let offset;
        match nodes {
            0 => offset = 0,
            1 => offset = 0,
            _ => offset = 1,
        }
        let lc = (2 * nodes - offset) as f64;
        let scaled = r * link_scale;
        let width = scaled / lc;
        let step = scaled / (nodes as f64);
        return (width, step, step * 0.5);
    }
}
