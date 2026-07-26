use std::mem;

use wasm_bindgen::prelude::*;

use crate::{
    Point,
    diagram::DiagramOpt,
    node::Node,
    square::Square,
    utils::{compute_line_box, create_container_id, get_angle, get_distance, get_xy},
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
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
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

pub struct DrawData {
    pub line_width: f64,

    pub bundle_side: f64,
    pub bundles: Vec<Point>,
    pub links: Vec<(Point, Point)>,
    pub animations: Vec<AnimationLink>,
    pub index: Square,
}
pub struct LinkContainer {
    pub links: Vec<Link>,
    pub bundles: Vec<Bundle>,
    pub draw_data: Option<DrawData>,
    pub id: u64,
}

impl LinkContainer {
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
        let angle = get_angle(src_p.x, src_p.y, dst_p.x, dst_p.y);
        let north = angle + 90.0;
        let south = north + 180.0;
        let r = src.layout.smallest_side(&dst.layout) * 0.5;

        let mut ne = get_xy(src_p.x, src_p.y, r, north);
        let mut nw = get_xy(dst_p.x, dst_p.y, r, north);
        let se = get_xy(src_p.x, src_p.y, r, south);
        let sw = get_xy(dst_p.x, dst_p.y, r, south);
        let idx = Square::from(compute_line_box(&ne, [&nw, &se, &sw]));
        let (width, step, init_step) = self.compute_line_width(opt.link_scale, r, self.links.len());
        ne = get_xy(ne.x, ne.y, r, angle + 180.0);
        nw = get_xy(nw.x, nw.y, r, angle);

        let mut animations = Vec::new(); // no way to know how big this will be :/
        let mut bundles = Vec::with_capacity(self.bundles.len());
        let mut links = Vec::with_capacity(self.links.len());
        ne = get_xy(ne.x, ne.y, r, angle + 180.0);
        nw = get_xy(nw.x, nw.y, r, angle);
        let bundle_side = r * 2.0 * opt.link_scale;

        for (i, link) in self.links.iter().enumerate() {
            let inc_by = init_step + step * (i as f64);
            let start = get_xy(ne.x, ne.y, inc_by, south);
            let end = get_xy(nw.x, nw.y, inc_by, south);
            let clink = (start, end);
            self.compute_animation(link, &clink, &mut animations, width, north, south);

            links.push(clink);
        }
        self.compute_bunlde_points(&src_p, &dst_p, self.bundles.len(), &mut bundles);

        self.draw_data = Some(DrawData {
            line_width: width,
            bundle_side,
            bundles,
            links,
            animations,
            index: idx,
        })
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
