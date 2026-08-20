use std::{cell::RefCell, rc::Weak};

use js_sys::Array;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{
    DiagramOpt, ElementOpt, LabelPosition, Point, Transform,
    bsp::ScreenSlot,
    constants::CANVAS_ERROR,
    diagram::DiagramCore,
    imgcache::ImgCache,
    link::{DrawData, Link, LinkContainer},
    node::Node,
    render::{BuildRender, CoreRender},
    square::Square,
    utils::{angle_check, angle_fix, get_angle, get_xy},
};

pub fn unpack_canvas(c: HtmlCanvasElement) -> Result<CanvasRenderingContext2d, JsValue> {
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
    canvas: HtmlCanvasElement,
}

impl BuildRender for CanvasRender {
    fn new(
        canvas: HtmlCanvasElement,
        diagram: Weak<RefCell<DiagramCore>>,
    ) -> Result<Box<dyn CoreRender>, JsValue> {
        let ctx = unpack_canvas(canvas.clone())?;
        ctx.set_text_align(&"center");
        ctx.set_text_baseline(&"middle");
        let d = unsafe { diagram.upgrade().unwrap_unchecked() };
        let frame_tick = d.borrow().render_ops.frame_tick;
        let dashes = animation_dash(&d.borrow().render_ops.animation_dashes);
        Ok(Box::new(Self {
            frame_tick,
            diagram,
            ctx,
            dashes,
            canvas,
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
    fn get_width_height(&self) -> (f32, f32) {
        (self.canvas.width() as f32, self.canvas.height() as f32)
    }

    fn render(&self) -> Result<(), JsValue> {
        let context = &self.ctx;
        context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
        {
            let (width, height) = self.get_width_height();
            context.clear_rect(0.0, 0.0, width as f64, height as f64);
        }
        let d = unsafe { self.diagram.upgrade().unwrap_unchecked() };
        let diagram = &*d.borrow();
        let opt = &diagram.render_ops;
        let t = *&*diagram.transform.borrow();

        context.set_global_alpha(1.0);
        self.draw_grid(opt)?;
        context.set_transform(t.k as f64, 0.0, 0.0, t.k as f64, t.x as f64, t.y as f64)?;
        context.set_line_dash_offset(self.frame_tick as f64);

        let cache = &diagram.img_cache;
        context.set_font(&opt.font_family);

        let node_vec = &diagram.nodes.borrow();
        let boxes_vec = &diagram.boxes.borrow();
        let link_vec = &diagram.links.borrow();
        for node in boxes_vec.iter().rev() {
            self.draw_node(node, diagram, opt, cache, false)?;
        }

        for link in link_vec.iter().rev() {
            self.draw_link(link, diagram, opt, cache, &t)?;
        }
        for (node, _) in node_vec.iter().rev() {
            self.draw_node(node, diagram, opt, cache, false)?;
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
            let o = diagram.get_opt(node.opt);
            self.draw_box(&node.layout, opt, o, true, cache)?;
            self.draw_node_text_highlight(&node.layout, &node.label, o, opt)?;
        }

        for set in &highlights.links {
            let link = &link_vec[set.link];
            let (src, dst, _) = &link.draw_data.links[set.element];
            let width = link.draw_data.line_width * opt.highlight_scale;

            self.draw_line(&src, &dst, width, highight_color);
        }
        for set in &highlights.bundles {
            let link = &link_vec[set.link];
            let bundle = &link.ls.bundles[set.element];
            let target = link.draw_data.bundle_draw_box(set.element);
            let o = diagram.get_opt(bundle.opt);
            self.draw_box(&target, opt, o, true, cache)?;
            self.draw_node_text_highlight(&target, &bundle.label, o, opt)?;
        }
        for id in &highlights.nodes {
            let node = &node_vec[*id].0;
            let o = diagram.get_opt(node.opt);
            self.draw_box(&node.layout, opt, o, true, cache)?;
            self.draw_node_text_highlight(&node.layout, &node.label, o, opt)?;
        }

        Ok(())
    }

    // this is a stub function required for the render behavior
    fn update(&self, _target: ScreenSlot, _p: &Point) {}
    fn clear(&self) {}
}

impl CanvasRender {
    fn draw_grid(&self, dops: &DiagramOpt) -> Result<(), JsValue> {
        let opt = match &dops.grid_opt {
            Some(o) => o,
            None => return Ok(()),
        };
        let (width, height) = self.get_width_height();
        let grid_size = opt.grid_size;
        let mut slot = 0;
        let grid_slots = opt.grid_slots;
        let divider_width = opt.grid_divider_width;
        let line_width = opt.grid_line_width;
        let color = &opt.grid_color;
        let x_offset = (width % grid_size as f32) * 0.5;
        let y_offset = (height % grid_size as f32) * 0.5;

        for i in (0..width as u32).step_by(grid_size as usize) {
            slot += 1;
            let x = i as f32 + x_offset;
            let pos = slot % grid_slots;
            let src = Point::new(x, 0.0);
            let dst = Point::new(x, height as f32);
            self.draw_line(
                &src,
                &dst,
                match pos == 0 {
                    false => divider_width,
                    true => line_width,
                } as f32,
                color,
            );
        }
        slot = 0;
        for i in (grid_size..height as u32).step_by(grid_size as usize) {
            slot += 1;
            let pos = slot % grid_slots;
            let y = i as f32 + y_offset;
            let src = Point::new(0.0, y);
            let dst = Point::new(width, y);
            self.draw_line(
                &src,
                &dst,
                match pos == 0 {
                    false => divider_width,
                    true => line_width,
                } as f32,
                color,
            );
        }

        Ok(())
    }
    fn draw_node_text_highlight(
        &self,
        square: &Square,
        text: &String,
        o: &ElementOpt,
        opt: &DiagramOpt,
    ) -> Result<(), JsValue> {
        if text.is_empty() {
            return Ok(());
        }
        let target = square.scale(opt.highlight_scale);
        let ctx = &self.ctx;
        let (w, h) = self.get_text_size(text)?;
        let x = target.x as f64 + (target.width * 0.5) as f64 - w * 0.5;
        let y = match o.label_position {
            LabelPosition::Top => square.y as f64 - h,
            LabelPosition::Bottom => square.max_y() as f64,
            LabelPosition::Center => (square.y + square.height * 0.5) as f64 - h * 0.5,
        };
        ctx.set_fill_style_str(&opt.highlight_color);
        ctx.fill_rect(x, y, w, h);
        Ok(())
    }
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
    pub fn compute_link_text(
        &self,
        src: &Point,
        dst: &Point,
        text: &String,
        height: f32,
        angle: f32,
        o: &ElementOpt,
    ) -> Result<(Point, f32), JsValue> {
        let meta = self.ctx.measure_text(&text)?;
        let font_height =
            (meta.actual_bounding_box_ascent() + meta.actual_bounding_box_descent()) as f32;
        let center = src.get_center(dst);
        let scale = height / font_height as f32;
        let r = (height + font_height * 0.5) * scale as f32;

        let new_angle = angle_fix(angle);
        let p = match o.label_position {
            // _ => center.scale(1.0 / scale),
            LabelPosition::Center => center,
            LabelPosition::Bottom => get_xy(center.x, center.y, r, new_angle + 90.0),
            LabelPosition::Top => get_xy(center.x, center.y, r, new_angle + 270.0),
        };
        Ok((p, scale * 0.5))
    }

    pub fn draw_link_text(
        &self,
        src: &Point,
        dst: &Point,
        o: &ElementOpt,
        text: &String,
        height: f32,
        angle: f32,
        opt: &DiagramOpt,
        t: &Transform,
    ) -> Result<(), JsValue> {
        if text.is_empty() {
            return Ok(());
        }
        let (_, font_height) = self.get_text_size(text)?;
        if font_height == 0.0 {
            return Ok(());
        }
        // don't alow rotation beyond 90 degrees.. as it will invert the text!

        let (p, scale) = self.compute_link_text(src, dst, text, height, angle, o)?;

        let full_scale = scale * t.k;

        let x = p.x * t.k + t.x;
        let y = p.y * t.k + t.y;
        let ctx = &self.ctx;

        let new_angle = angle_fix(angle);
        let angle = (new_angle).to_radians();
        let k = (full_scale * angle.cos()) as f64;
        let r = (full_scale * angle.sin()) as f64;
        ctx.set_transform(k as f64, r, -r, k as f64, x as f64, y as f64)?;

        self.draw_text(0 as f64, 0 as f64, text, &opt.font_color)?;
        ctx.set_transform(t.k as f64, 0.0, 0.0, t.k as f64, t.x as f64, t.y as f64)?;

        Ok(())
    }
    pub fn draw_sublink(
        &self,
        ld: &Link,
        diagram: &DiagramCore,
        opt: &DiagramOpt,
        t: &Transform,
        data: &DrawData,
        i: usize,
        width: f32,
        angle: f32,
    ) -> Result<(), JsValue> {
        let (a, b, _) = &data.links[i];
        let o = diagram.get_opt(ld.opt);
        self.draw_line(a, b, width, &o.color);
        self.draw_link_text(a, b, o, &ld.label, width, angle, opt, t)?;
        Ok(())
    }
    pub fn draw_link(
        &self,
        link: &LinkContainer,
        diagram: &DiagramCore,
        opt: &DiagramOpt,
        cache: &ImgCache,
        t: &Transform,
    ) -> Result<(), JsValue> {
        let ctx = &self.ctx;

        let data = &link.draw_data;
        if data.links.len() == 0 {
            return Ok(());
        }
        let angle = {
            let (a, b, _) = &data.links[0];
            get_angle(a.x, a.y, b.x, b.y)
        };
        let width = data.line_width;
        if angle_check(angle) {
            for (i, ld) in link.ls.links.iter().enumerate() {
                self.draw_sublink(ld, diagram, opt, t, data, i, width, angle)?;
            }
        } else {
            for (i, ld) in link.ls.links.iter().rev().enumerate() {
                self.draw_sublink(ld, diagram, opt, t, data, i, width, angle)?;
            }
        }

        ctx.set_line_dash(&self.dashes)?;
        for (a, b, width) in &data.animations {
            self.draw_line(a, b, *width, &opt.animation_color);
        }
        ctx.set_line_dash(&Array::new())?;

        for (i, bundle) in link.ls.bundles.iter().enumerate() {
            let target = data.bundle_draw_box(i);
            self.draw_box(&target, opt, diagram.get_opt(bundle.opt), false, &cache)?;

            let o = diagram.get_opt(bundle.opt);
            let (x, y, _, _) = self.text_pos(&target, o, &bundle.label)?;
            self.draw_text(x as f64, y as f64, &bundle.label, &opt.font_color)?
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
        opts: &DiagramOpt,
        cache: &ImgCache,
        highlight: bool,
    ) -> Result<(), JsValue> {
        let o = diagram.get_opt(node.opt);
        self.draw_box(&node.layout, &diagram.render_ops, o, highlight, &cache)?;
        if node.label.is_empty() {
            return Ok(());
        }

        let (x, y, _, _) = self.text_pos(&node.layout, o, &node.label)?;
        self.draw_text(x as f64, y as f64, &node.label, &opts.font_color)
    }

    fn get_text_size(&self, text: &String) -> Result<(f64, f64), JsValue> {
        let meta = self.ctx.measure_text(&text)?;
        let w = meta.actual_bounding_box_left() + meta.actual_bounding_box_right();
        let height = meta.actual_bounding_box_ascent() + meta.actual_bounding_box_descent();
        Ok((w, height))
    }
    fn text_pos(
        &self,
        l: &Square,
        o: &ElementOpt,
        text: &String,
    ) -> Result<(f64, f64, f64, f64), JsValue> {
        let (width, height) = self.get_text_size(text)?;
        let x = l.x + l.width * 0.5;
        let h = (height * 0.5) as f32;

        // We always center on the x axis
        let y = match o.label_position {
            LabelPosition::Top => l.y - h,
            LabelPosition::Bottom => l.max_y() + h,
            LabelPosition::Center => l.y + l.height * 0.5,
        };
        Ok((x as f64, y as f64, height, width))
    }
}
