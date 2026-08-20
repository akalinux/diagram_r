use wasm_bindgen::prelude::*;

use crate::{
    DiagramOpt, Point,
    bsp::LookupPointResult,
    constants::ZERO_POINT,
    node::Node,
    square::Square,
    utils::{compute_line_box, full_box_from, get_xy, inside_box},
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
    pub arc: Option<Point>,
}

#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug, PartialEq)]
pub struct LinkSet {
    pub src: usize,
    pub dst: usize,
    pub links: Vec<Link>,
    pub bundles: Vec<Bundle>,
}

#[wasm_bindgen]
impl LinkSet {
    #[wasm_bindgen(constructor)]
    pub fn new(links: Vec<Link>, bundles: Vec<Bundle>, src: usize, dst: usize) -> Self {
        Self {
            src,
            dst,
            links,
            bundles,
        }
    }
}

pub fn compute_animation_width(link_scale: f32, r: f32, links: usize) -> (f32, f32, f32) {
    //let lc = self.compute_node_scale(nodes) as f32;
    let offset = match links {
        1 => 0,
        _ => 1,
    };
    let lc = (2 * links - offset) as f32;
    let scaled = r * link_scale;
    let width = scaled / lc;
    let step = scaled / (links as f32);
    return (width, step, step * 0.5);
}
pub type LineAnimation = Vec<(Point, Point, Option<Point>, f32)>;
pub type ComputedLink = (Point, Point, Option<Point>);
pub fn compute_animation(
    link: &Link,
    clink: &ComputedLink,
    width: f32,
    angle_north: f32,
) -> Option<LineAnimation> {
    match link.animation {
        Animation::Both => {
            let (aw, _, init_step) = compute_animation_width(1.0, width, 2);

            let ne = get_xy(clink.0.x, clink.0.y, init_step, angle_north);
            let d = ne.get_move_distance(&clink.0);
            let arc = match &clink.2 {
                Some(p) => Some(p.sub_distance(&d)),
                None => None,
            };
            let nw = clink.1.sub_distance(&d);
            let mut res = Vec::with_capacity(2);
            res.push((ne, nw, arc, aw));

            res.push((
                clink.0.add_distance(&d),
                clink.1.add_distance(&d),
                match &clink.2 {
                    Some(p) => Some(p.add_distance(&d)),
                    None => None,
                },
                aw,
            ));
            Some(res)
        }
        Animation::ToSrc => {
            let (aw, _, _) = compute_animation_width(1.0, width, 1);
            Some(Vec::from([(clink.1, clink.0, clink.2, aw)]))
        }
        Animation::ToDst => {
            let (aw, _, _) = compute_animation_width(1.0, width, 1);

            Some(Vec::from([(clink.0, clink.1, clink.2, aw)]))
        }
        _ => None,
    }
}

fn get_line_width(total_links: usize, smallest_side: f32, link_scale: f32) -> (f32, f32, f32) {
    let full_width = smallest_side * link_scale;
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

        let d = src.get_move_distance(dst);

        for bundle in &self.bundles {
            points.push(src.add_distance(&d.scale(bundle.pos)));
        }
        points
    }
    pub fn build_draw_data(&self, src: &Node, dst: &Node, opt: &DiagramOpt) -> DrawData {
        let src_p = src.layout.get_center();
        let dst_p = dst.layout.get_center();
        let smallest_side = src.layout.smallest_side(&dst.layout);
        let r = smallest_side * 0.5;
        let ((mut nw, mut ne, sw, se), (mut d, north)) = full_box_from(&src_p, &dst_p, r);

        let idx = Square::from(compute_line_box(&ne, [&nw, &se, &sw]));
        let (width, inital_scale, scale) =
            get_line_width(self.links.len(), smallest_side, opt.link_scale);

        let mut links = Vec::with_capacity(self.links.len());
        d = d.scale(2.0);
        let init = d.scale(inital_scale);
        let chunk = d.scale(scale);
        //log(&format!("{:?},{:?}, {},{}", init, d, width, inital_scale));
        nw = nw.add_distance(&init);
        ne = ne.add_distance(&init);
        for (i, link) in self.links.iter().enumerate() {
            let ix = i as f32;
            let start = nw.add_distance(&chunk.scale(ix));
            let end = ne.add_distance(&chunk.scale(ix));
            let animation = compute_animation(link, &(start, end, None), width * 0.5, north);

            links.push((start, end, None, animation));
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
}

#[wasm_bindgen]
impl Link {
    #[wasm_bindgen(constructor)]
    pub fn new(opt: usize, label: String, animation: Animation, arc: Option<Point>) -> Self {
        Self {
            opt,
            label,
            animation,
            arc,
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

pub type SubLink = (Point, Point, Option<Point>, Option<LineAnimation>);
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
            // FIXME!
            if let Some(list) = &mut link.3 {
                for (src, dst, arc, _) in list {
                    *src = src.add_distance(distance);
                    *dst = dst.add_distance(distance);
                    *arc = match arc {
                        Some(p) => Some(p.add_distance(distance)),
                        None => None,
                    };
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
    id: usize,
}

impl LinkContainer {
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
            let lp = &dd.links[i];
            let (pb, _) = full_box_from(&lp.0, &lp.1, dd.line_width * 0.5);
            if inside_box(&pb, p) {
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
                // TODO
                let (a, b, _, _) = dd.links[*i];
                Point {
                    x: (a.x + b.x) / 2.0,
                    y: (a.y + b.y) / 2.0,
                }
            }
            LookupPointResult::Bundle((i, _)) => dd.bundles[*i],
            _ => ZERO_POINT,
        }
    }
}
