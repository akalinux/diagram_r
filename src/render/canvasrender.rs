use std::{cell::RefCell, rc::Weak};

use js_sys::Array;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{
    DiagramOpt, ElementOpt, LabelPosition, Point,
    bsp::ScreenSlot,
    constants::CANVAS_ERROR,
    diagram::DiagramCore,
    imgcache::ImgCache,
    link::LinkContainer,
    node::Node,
    render::{BuildRender, CoreRender},
    square::Square,
    utils::{get_angle, get_xy},
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
    frame_tick: f32,
    dashes: Array,
}

impl BuildRender for CanvasRender {
    fn new(
        canvas: &HtmlCanvasElement,
        diagram: Weak<RefCell<DiagramCore>>,
    ) -> Result<Box<dyn CoreRender>, JsValue> {
        let ctx = unpack_canvas(canvas)?;
        ctx.set_text_align(&"center");
        ctx.set_text_baseline(&"middle");
        let d = unsafe { diagram.upgrade().unwrap_unchecked() };
        ctx.set_font(&d.borrow().render_ops.font_family);
        let frame_tick = d.borrow().render_ops.frame_tick;
        let dashes = animation_dash(&d.borrow().render_ops.animation_dashes);
        Ok(Box::new(Self {
            frame_tick,
            diagram,
            ctx,
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
        context.set_line_dash_offset(self.frame_tick as f64);
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
        context.set_global_alpha(opt.highlight_alpha as f64);
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

    // this is a stub function required for the render behavior
    fn update(&self, _target: ScreenSlot, _p: &Point) {}
    fn clear(&self) {}
}

impl CanvasRender {
    fn draw_line(&self, src: &Point, dst: &Point, width: f32, color: &String) {
        let ctx = &self.ctx;
        ctx.begin_path();
        ctx.set_line_width(width as f64);
        ctx.set_stroke_style_str(&color);
        ctx.move_to(src.x as f64, src.y as f64);
        ctx.line_to(dst.x as f64, dst.y as f64);
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

            let (x, y, w, h) = target.scale(opt.highlight_scale).render_points64();
            ctx.clear_rect(x, y, w, h);
            ctx.fill_rect(x, y, w, h);
        } else {
            let (x, y, w, h) = target.render_points64();
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
    pub fn draw_link_text(
        &self,
        src: &Point,
        dst: &Point,
        o: &ElementOpt,
        text: &String,
        line_width: f32,
    ) -> Result<(), JsValue> {
        if text.is_empty() {
            return Ok(());
        }
        let (_, height) = self.get_text_size(text)?;
        if height == 0.0 {
            return Ok(());
        }
        // don't alow rotation beyond 90 degrees.. as it will invert the text!
        let angle = get_angle(src.x, src.y, dst.x, dst.y) % 90.0;

        let center = src.get_center(dst);
        let p = match o.label_position {
            LabelPosition::Center => center,
            LabelPosition::Bottom => get_xy(center.x, center.y, line_width * 0.5, angle + 90.0),
            LabelPosition::Top => get_xy(center.x, center.y, line_width * 0.5, angle + 270.0),
        };

        let scale = (line_width * 0.5) as f64 / height;
        let ctx = &self.ctx;
        ctx.save();
        match ctx.rotate(angle.to_radians() as f64) {
            Err(e) => {
                ctx.restore();
                return Err(e);
            }
            _ => (),
        };

        match ctx.scale(scale, scale) {
            Err(e) => {
                ctx.restore();
                return Err(e);
            }
            _ => (),
        };
        match self.draw_text(p.x as f64, p.y as f64, text, &o.color) {
            Err(e) => {
                ctx.restore();
                return Err(e);
            }
            _ => (),
        }
        ctx.restore();

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
            let o = diagram.get_opt(ld.opt);
            self.draw_line(a, b, width, &o.color);
            self.draw_link_text(a, b, o, &ld.label, data.line_width)?;
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
    pub fn draw_text(&self, x: f64, y: f64, text: &String, color: &String) -> Result<(), JsValue> {
        let ctx = &self.ctx;
        ctx.set_fill_style_str(color);
        ctx.fill_text(text, x, y)
    }
    pub fn draw_node(
        &self,
        node: &Node,
        diagram: &DiagramCore,
        cache: &ImgCache,
        highlight: bool,
    ) -> Result<(), JsValue> {
        let o = diagram.get_opt(node.opt);
        self.draw_box(&node.layout, &diagram.render_ops, o, highlight, &cache)?;
        if node.label.is_empty() {
            return Ok(());
        }

        let (x, y, _) = self.text_pos(&node.layout, o, &node.label)?;

        self.draw_text(x as f64, y as f64, &node.label, &o.color)
    }

    fn get_text_size(&self, text: &String) -> Result<(f64, f64), JsValue> {
        let meta = self.ctx.measure_text(&text)?;
        let w = meta.width();
        let height = meta.actual_bounding_box_descent() + meta.actual_bounding_box_descent();
        Ok((w, height))
    }
    fn text_pos(
        &self,
        l: &Square,
        o: &ElementOpt,
        text: &String,
    ) -> Result<(f64, f64, f64), JsValue> {
        let (width, height) = self.get_text_size(text)?;
        let x = width as f32 * 0.5;
        let h = (height * 0.5) as f32;

        // We always center on the x axis
        let y = match o.label_position {
            LabelPosition::Top => l.max_y() - h,
            LabelPosition::Bottom => l.max_y() + h,
            LabelPosition::Center => l.y + l.height * 0.5,
        };
        Ok((x as f64, y as f64, height))
    }
}
