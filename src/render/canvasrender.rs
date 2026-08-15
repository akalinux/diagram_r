use std::{cell::RefCell, rc::Weak};

use js_sys::Array;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{
    DiagramOpt, ElementOpt, Point,
    bsp::ScreenSlot,
    constants::CANVAS_ERROR,
    diagram::DiagramCore,
    imgcache::ImgCache,
    link::{DrawData, LinkContainer},
    node::Node,
    render::{BuildRender, CoreRender},
    square::Square,
};

pub fn unpack_canvas(c: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, JsValue> {
    match c.get_context("2d") {
        Ok(o) => match o {
            Some(obj) => match obj.dyn_into::<web_sys::CanvasRenderingContext2d>() {
                Ok(context) => Ok(context),
                Err(_) => Err(JsValue::from(CANVAS_ERROR)),
            },
            None => Err(JsValue::from(CANVAS_ERROR)),
        },
        Err(e) => Err(e),
    }
}

pub struct CanvasRender {
    diagram: Weak<RefCell<DiagramCore>>,
    ctx: CanvasRenderingContext2d,
    frame_tick: f64,
    link_data: RefCell<Option<Vec<DrawData>>>,
    dashes: Array,
}

impl BuildRender for CanvasRender {
    fn new(
        canvas: &HtmlCanvasElement,
        diagram: Weak<RefCell<DiagramCore>>,
    ) -> Result<Box<dyn CoreRender>, JsValue> {
        let ctx = unpack_canvas(canvas)?;
        let d = unsafe { diagram.upgrade().unwrap_unchecked() };
        let frame_tick = d.borrow().render_ops.frame_tick;
        let dashes = animation_dash(&d.borrow().render_ops.animation_dashes);
        Ok(Box::new(Self {
            frame_tick,
            diagram,
            ctx,
            link_data: RefCell::new(None),
            dashes,
        }))
    }
}
fn animation_dash(src: &Vec<f64>) -> Array {
    let res = Array::new();
    for dash in src {
        res.push(&JsValue::from_f64(*dash));
    }
    res
}
impl CoreRender for CanvasRender {
    fn render(&self) -> Result<(), JsValue> {
        let context = &self.ctx;
        context.set_global_alpha(1.0);
        context.set_line_dash_offset(self.frame_tick);
        let d = unsafe { self.diagram.upgrade().unwrap_unchecked() };
        let diagram = &*d.borrow();

        let cache = &diagram.img_cache;
        let opt = &diagram.render_ops;
        let node_vec = &diagram.nodes.borrow();
        let boxes_vec = &diagram.boxes.borrow();
        let link_vec = &diagram.links.borrow();
        for node in boxes_vec.iter() {
            self.draw_node(node, diagram, cache, false)?;
        }

        for link in link_vec.iter() {
            self.draw_link(link, diagram, opt, cache)?;
        }
        for (node, _) in node_vec.iter() {
            self.draw_node(node, diagram, cache, false)?;
        }

        let h = diagram.highlights.borrow();
        let highlights = match h.as_ref() {
            Some(h) => h,
            // # -- STOP HERE IF WE HAVE NOTHING TO HIGHLIGHT! -- #
            None => return Ok(()),
        };

        let highight_color = &opt.highlight_color;
        context.set_global_alpha(opt.highlight_alpha);
        for id in &highlights.boxes {
            let node = &boxes_vec[*id];
            self.draw_box(&node.layout, opt, diagram.get_opt(node.opt), true, cache)?;
        }

        for set in &highlights.links {
            let link = &link_vec[set.link];
            let src = node_vec[link.ls.src].0.layout.get_center();
            let dst = node_vec[link.ls.dst].0.layout.get_center();
            let width = link.draw_data.line_width * opt.highlight_scale;

            self.draw_line(&src, &dst, width, highight_color);
        }
        for set in &highlights.bundles {
            let link = &link_vec[set.link];
            let bundle = &link.ls.bundles[set.element];
            let target = link.draw_data.bundle_draw_box(set.element);
            self.draw_box(&target, opt, diagram.get_opt(bundle.opt), true, cache)?;
        }
        for id in &highlights.nodes {
            let node = &node_vec[*id].0;
            self.draw_box(&node.layout, opt, diagram.get_opt(node.opt), true, cache)?;
        }

        Ok(())
    }

    fn update(&self, target: ScreenSlot, p: &Point) {
        let mut links = self.link_data.borrow_mut();
        let (id, data) = match (target, links.as_mut()) {
            (ScreenSlot::Link(id), Some(data)) => (id, data),
            _ => return,
        };
        data[id].move_distance(p);
    }
    fn clear(&self) {}
}

impl CanvasRender {
    fn draw_line(&self, src: &Point, dst: &Point, width: f64, color: &String) {
        let ctx = &self.ctx;
        ctx.begin_path();
        ctx.set_line_width(width);
        ctx.set_stroke_style_str(&color);
        ctx.move_to(src.x, src.y);
        ctx.line_to(dst.x, dst.y);
        ctx.close_path();
        ctx.stroke();
    }
    pub fn draw_box(
        &self,
        target: &Square,
        opt: &DiagramOpt,
        o: &ElementOpt,
        highlight: bool,
        cache: &ImgCache,
    ) -> Result<(), JsValue> {
        let ctx = &self.ctx;
        if highlight {
            ctx.set_fill_style_str(&opt.highlight_color);

            let (x, y, w, h) = target.scale(opt.highlight_scale).render_points();
            ctx.clear_rect(x, y, w, h);
            ctx.fill_rect(x, y, w, h);
        } else {
            let (x, y, w, h) = target.render_points();
            if let Some(res) = cache.load_img(&o.img)
                && let Ok(img) = res
            {
                ctx.draw_image_with_html_image_element_and_dw_and_dh(&img, x, y, w, h)?;
            } else {
                ctx.set_fill_style_str(&o.color);
                ctx.fill_rect(x, y, w, h);
            }
        }
        Ok(())
    }
    pub fn draw_link(
        &self,
        link: &LinkContainer,
        diagram: &DiagramCore,
        opt: &DiagramOpt,
        cache: &ImgCache,
    ) -> Result<(), JsValue> {
        let ctx = &self.ctx;

        let data = &link.draw_data;
        let width = data.line_width;
        for (i, ld) in link.ls.links.iter().enumerate() {
            let (a, b) = &data.links[i];
            self.draw_line(a, b, width, &diagram.get_opt(ld.opt).color);
        }

        ctx.set_line_dash(&self.dashes)?;
        for (a, b, width) in &data.animations {
            self.draw_line(a, b, *width, &opt.animation_color);
        }
        ctx.set_line_dash(&Array::new())?;

        for (i, bundle) in link.ls.bundles.iter().enumerate() {
            let target = data.bundle_draw_box(i);
            self.draw_box(&target, opt, diagram.get_opt(bundle.opt), false, &cache)?;
        }
        Ok(())
    }
    pub fn draw_node(
        &self,
        node: &Node,
        diagram: &DiagramCore,
        cache: &ImgCache,
        highlight: bool,
    ) -> Result<(), JsValue> {
        self.draw_box(
            &node.layout,
            &diagram.render_ops,
            diagram.get_opt(node.opt),
            highlight,
            &cache,
        )
    }
}
