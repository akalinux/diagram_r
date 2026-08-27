use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use js_sys::Array;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{
    DiagramOpt, ElementOpt, LabelPosition, Point, Transform,
    bsp::ScreenSlot,
    constants::{CANVAS_ERROR, HALF, R_90, R_270},
    diagram::DiagramCore,
    imgcache::ImgCache,
    link::{ArcType, LineAnimation, LinkContainer},
    node::Node,
    render::{BuildRender, CoreRender, rendertimer::FrameTimer},
    square::Square,
    utils::normalize_rad,
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
    frame_tick: RefCell<f64>,
    // total and tick per frame
    total_and_offset: (f64, f64),
    dashes: Array,
    canvas: HtmlCanvasElement,
    frame_timer: RefCell<Option<Rc<RefCell<FrameTimer>>>>,
    animate: RefCell<bool>,
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
        ctx.set_font(&d.borrow().render_ops.font_family);
        let mut total = 0.0;
        for i in &d.borrow().render_ops.animation_dashes {
            total += *i as f64;
        }
        // work around for js doubling when just 1 is applied
        if d.borrow().render_ops.animation_dashes.len() == 1 {
            total *= 2.0;
        }
        let offset = total / d.borrow().render_ops.frame_rate as f64;
        let dashes = animation_dash(&d.borrow().render_ops.animation_dashes);
        Ok(Box::new(Self {
            total_and_offset: (total, offset),
            frame_tick: RefCell::new(total),
            diagram,
            ctx,
            dashes,
            canvas,
            frame_timer: RefCell::new(None),
            animate: RefCell::new(false),
        }))
    }
}
fn animation_dash(src: &Vec<i32>) -> Array {
    let res = Array::new();
    for dash in src {
        res.push(&JsValue::from(*dash));
    }
    res
}
impl CoreRender for CanvasRender {
    fn get_width_height(&self) -> (f32, f32) {
        (self.canvas.width() as f32, self.canvas.height() as f32)
    }

    fn render(&self) -> Result<(), JsValue> {
        self.animate.replace(false);
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
        context.set_line_dash_offset(*self.frame_tick.borrow() as f64);

        let cache = &diagram.img_cache;

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

        if opt.animate && *self.animate.borrow() {
            if self.frame_timer.borrow().is_none() {
                let ft = FrameTimer::new(self.diagram.clone())?;
                self.frame_timer.replace(Some(ft));
            }
        } else {
            if self.frame_timer.borrow().is_some() {
                self.frame_timer.replace(None);
            }
        }

        let h = diagram.highlights.borrow();
        let highlights = match h.as_ref() {
            Some(h) => h,
            // # -- STOP HERE IF WE HAVE NOTHING TO HIGHLIGHT! -- #
            None => return Ok(()),
        };

        context.set_global_alpha(opt.highlight_alpha as f64);
        for id in &highlights.boxes {
            let node = &boxes_vec[*id];
            let o = diagram.get_opt(node.opt);
            self.draw_box(&node.layout, opt, o, true, cache)?;
            self.draw_node_text_highlight(&node.layout, &node.label, o, opt)?;
        }

        for set in &highlights.links {
            let lc = &link_vec[set.link];

            self.draw_sublink(lc, set.element, diagram, opt, &t, true)?;
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
    fn update(&self, _target: ScreenSlot) {}
    fn clear(&self) {
        let context = &self.ctx;
        let _ = context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        {
            let (width, height) = self.get_width_height();
            context.clear_rect(0.0, 0.0, width as f64, height as f64);
        }
    }
    fn animate(&self) -> Result<(), JsValue> {
        let (total, offset) = self.total_and_offset;
        let tick = (*self.frame_tick.borrow() - offset) % total;

        self.frame_tick.replace(tick);
        self.render()
    }
}

impl CanvasRender {
    fn draw_grid(&self, dops: &DiagramOpt) -> Result<(), JsValue> {
        let opt = match &dops.grid_opt {
            Some(o) => o,
            None => return Ok(()),
        };
        let (width, height) = self.get_width_height();
        let grid_size = opt.grid_size;
        let grid_slots = opt.grid_slots;
        let divider_width = opt.grid_divider_width;
        let line_width = opt.grid_line_width;
        let color = &opt.grid_color;
        let x_offset = (width % grid_size as f32) * HALF;
        let y_offset = (height % grid_size as f32) * HALF;
        let y_scale = height / width;
        let mut slot = 0;
        let (mut p, mut pos);

        for i in (0..width as u32).step_by(grid_size as usize) {
            slot += 1;
            p = i as f32 + x_offset;
            pos = slot % grid_slots;
            let w = match pos == 0 {
                false => divider_width,
                true => line_width,
            };
            self.raw_line_draw(p, 0.0, p, height, w, color);
            p = i as f32 * y_scale + y_offset;
            self.raw_line_draw(0.0, p, width, p, w, color);
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
        let ctx = &self.ctx;

        let (x, y, w, h) = self.get_box_text_position(square, o, text)?;
        ctx.set_fill_style_str(&opt.highlight_color);
        ctx.fill_rect(x - w * HALF as f64, y - h * HALF as f64, w, h);
        Ok(())
    }
    fn raw_line_draw(&self, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: &String) {
        let ctx = &self.ctx;
        ctx.begin_path();
        ctx.set_line_width(width as f64);
        ctx.set_stroke_style_str(&color);
        ctx.move_to(x1 as f64, y1 as f64);
        ctx.line_to(x2 as f64, y2 as f64);

        ctx.stroke();
    }

    fn draw_line(&self, src: &Point, dst: &Point, width: f32, color: &String) {
        self.raw_line_draw(src.x, src.y, dst.x, dst.y, width, color);
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

            let (x, y, w, h) = target.render_points64();
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
    pub fn get_link_text_point_and_scale(
        &self,
        src: &Point,
        dst: &Point,
        height: f32,
        new_rad: f32,
        o: &ElementOpt,
        font_height: f32,
    ) -> Result<(Point, f32), JsValue> {
        let center = src.get_center(dst);
        let scale = height / font_height as f32;
        let r = height * 0.75;

        let p = match o.label_position {
            // _ => center.scale(1.0 / scale),
            LabelPosition::Center => center,
            LabelPosition::Bottom => center.get_xy(r, new_rad + R_90),
            LabelPosition::Top => center.get_xy(r, new_rad + R_270),
        };

        Ok((p, scale * HALF))
    }

    pub fn draw_link_text(
        &self,
        src: &Point,
        dst: &Point,
        o: &ElementOpt,
        text: &String,
        line_width: f32,
        new_rad: f32,
        opt: &DiagramOpt,
        t: &Transform,
        highlight: bool,
    ) -> Result<(), JsValue> {
        if text.is_empty() {
            return Ok(());
        }

        let (fw, fh) = self.get_text_size(text)?;
        let font_height = fh as f32;
        let (p, scale) =
            self.get_link_text_point_and_scale(src, dst, line_width, new_rad, o, font_height)?;
        if highlight {
            let start = p.get_xy(fw as f32 * HALF * scale, new_rad);
            let end = p.add_distance(&start.get_move_distance(&p));
            self.draw_line(
                &start,
                &end,
                font_height as f32 * scale,
                &opt.highlight_color,
            );
            return Ok(());
        }

        let full_scale = scale * t.k;

        let x = p.x * t.k + t.x;
        let y = p.y * t.k + t.y;
        let ctx = &self.ctx;

        let k = (full_scale * new_rad.cos()) as f64;
        let r = (full_scale * new_rad.sin()) as f64;
        ctx.set_transform(k as f64, r, -r, k as f64, x as f64, y as f64)?;

        self.draw_text(0 as f64, 0 as f64, text, &opt.font_color)?;
        ctx.set_transform(t.k as f64, 0.0, 0.0, t.k as f64, t.x as f64, t.y as f64)?;

        Ok(())
    }

    fn draw_arc(&self, p: &Point, color: &String, width: f32) -> Result<(), JsValue> {
        let ctx = &self.ctx;
        ctx.begin_path();
        ctx.arc(
            p.x as f64,
            p.y as f64,
            (width * HALF) as f64,
            0.0,
            2.0 * std::f64::consts::PI,
        )?;
        ctx.set_fill_style_str(color);
        ctx.fill();
        //self.ctx.stroke();
        Ok(())
    }
    pub fn draw_sublink(
        &self,
        lc: &LinkContainer,
        i: usize,
        diagram: &DiagramCore,
        opt: &DiagramOpt,
        t: &Transform,
        highlight: bool,
    ) -> Result<(), JsValue> {
        let link = &lc.ls.links[i];
        let o = diagram.get_opt(link.opt);
        let color = match highlight {
            true => &opt.highlight_color,
            false => &o.color,
        };
        let text = &link.label;
        let dd = &lc.draw_data;
        let width = dd.line_width;
        let o = diagram.get_opt(link.opt);
        match &lc.ls.point {
            Some(arc) => match &arc.mode {
                ArcType::Arc => todo!("FIXME!"),
                ArcType::Joint => {
                    for el in 0..2 {
                        let id = i * 2 + el;
                        let (a, b, animations) = &dd.links[id];
                        if el == 0 {
                            self.draw_arc(b, color, width)?;
                        }
                        self.draw_link_line(
                            a, b, width, text, opt, highlight, animations, color, t, o,
                        )?;
                    }
                    Ok(())
                }
            },
            None => {
                let (a, b, animations) = &dd.links[i];
                self.draw_link_line(a, b, width, text, opt, highlight, animations, color, t, o)
            }
        }
    }
    fn draw_link_line(
        &self,
        a: &Point,
        b: &Point,
        width: f32,
        text: &String,
        opt: &DiagramOpt,
        highlight: bool,
        animations: &LineAnimation,
        color: &String,
        t: &Transform,
        o: &ElementOpt,
    ) -> Result<(), JsValue> {
        self.draw_line(a, b, width, color);
        match highlight {
            false => self.draw_link_animations(animations, &opt.animation_color)?,
            true => (),
        };
        let rad = a.get_radians(b);
        let (normalized_angle, _) = normalize_rad(rad);
        self.draw_link_text(a, b, o, text, width, normalized_angle, opt, t, highlight)
    }

    pub fn draw_link_animations(
        &self,
        animation: &LineAnimation,
        color: &String,
    ) -> Result<(), JsValue> {
        match animation {
            LineAnimation::Both(width, a, b, c, d) => {
                self.animate.replace(true);
                self.ctx.set_line_dash(&self.dashes)?;
                self.draw_line(a, b, *width, color);
                self.draw_line(c, d, *width, color);
                self.ctx.set_line_dash(&Array::new())?;
            }
            LineAnimation::Side(width, a, b) => {
                self.animate.replace(true);
                self.ctx.set_line_dash(&self.dashes)?;
                self.draw_line(a, b, *width, color);
                self.ctx.set_line_dash(&Array::new())?;
            }
            LineAnimation::None => (),
        }

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
        for i in 0..link.ls.links.len() {
            self.draw_sublink(link, i, diagram, opt, t, false)?;
        }
        let data = &link.draw_data;

        for (i, bundle) in link.ls.bundles.iter().enumerate() {
            let target = data.bundle_draw_box(i);
            self.draw_box(&target, opt, diagram.get_opt(bundle.opt), false, &cache)?;

            let o = diagram.get_opt(bundle.opt);
            let (x, y, _, _) = self.get_box_text_position(&target, o, &bundle.label)?;
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

        let (x, y, _, _) = self.get_box_text_position(&node.layout, o, &node.label)?;
        self.draw_text(x as f64, y as f64, &node.label, &opts.font_color)
    }

    fn get_text_size(&self, text: &String) -> Result<(f64, f64), JsValue> {
        let meta = self.ctx.measure_text(&text)?;
        let w = meta.width();
        //meta.font_bounding_box_ascent()
        let height = meta.actual_bounding_box_ascent() + meta.actual_bounding_box_descent();
        Ok((w, height))
    }
    fn get_box_text_position(
        &self,
        l: &Square,
        o: &ElementOpt,
        text: &String,
    ) -> Result<(f64, f64, f64, f64), JsValue> {
        let (width, height) = self.get_text_size(text)?;
        let x = l.x + l.width * HALF;
        let h = height as f32 * HALF;

        // We always center on the x axis
        let y = match o.label_position {
            LabelPosition::Top => l.y - h,
            LabelPosition::Bottom => l.max_y() + h,
            LabelPosition::Center => l.y + l.height * HALF,
        };

        Ok((x as f64, y as f64, width, height))
    }
}
