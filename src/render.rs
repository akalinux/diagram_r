use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
pub mod event_watcher;
pub mod size_watcher;
pub mod timeout;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
pub mod targets;

use crate::{
    ElementOpt, Point,
    bsp::LookupPointResult,
    diagram::{DiagramCore, DiagramOpt, NodeCanvasTarget},
    imgcache::ImgCache,
    link::LinkContainer,
    node::Node,
    render::{targets::Targets, timeout::Timeout},
    square::Square,
};

pub struct Render {
    pub diagram: Weak<RefCell<DiagramCore>>,
    this: Weak<RefCell<Self>>,
    targets: Option<Targets>,
    timeout: RefCell<Option<Timeout>>,
    frame_tick: f64,
    current_target: RefCell<LookupPointResult>,
}

impl Render {
    pub fn new() -> Rc<RefCell<Self>> {
        let res = Self {
            diagram: Weak::new(),
            this: Weak::new(),
            targets: None,
            frame_tick: 0.0,
            timeout: RefCell::new(None),
            current_target: RefCell::new(LookupPointResult::NoMatch),
        };
        let rc = Rc::new(RefCell::new(res));
        rc.borrow_mut().this = Rc::downgrade(&rc);

        rc
    }
    pub fn on_img(&self, cache: &ImgCache) {
        if unsafe { self.diagram.upgrade().unwrap_unchecked() }
            .borrow()
            .render_ops
            .bulk_img_update
        {
            if !cache.is_done() {
                return;
            }
        }
        let _ = self.render();
    }

    pub fn on_mouse_down(&self, p: &Point) {
        self.clear_timeout();
    }
    pub fn on_mouse_up(&self, p: &Point) {
        self.clear_timeout();
    }
    pub fn on_mouse_enter(&self, p: &Point) {
        self.clear_timeout();
    }
    pub fn on_mouse_leave(&self, p: &Point) {
        self.clear_timeout();
    }
    pub fn on_mouse_move(&self, p: &Point) {
        self.clear_timeout();
    }
    pub fn on_mouse_wheel(&self, delta: f64) {
        self.clear_timeout();
    }
    pub fn mount(&mut self, width: u32, height: u32, id: String) -> Result<(), JsValue> {
        let targets = Targets::new(
            width,
            height,
            unsafe { self.this.upgrade().unwrap_unchecked() },
            id,
        )?;

        let d = unsafe { self.diagram.upgrade().unwrap_unchecked() };
        let diagram = d.borrow();
        let ops = &diagram.render_ops;
        targets.animations.set_line_dash(&ops.animation_dash())?;
        targets.boxes.set_font(&ops.font_family);
        targets.nodes.set_font(&ops.font_family);
        targets.links.set_font(&ops.font_family);
        targets.highlight.set_font(&ops.font_family);
        targets.highlight.set_global_alpha(ops.highlight_alpha);

        self.targets = Some(targets);
        Ok(())
    }
    pub fn unmount(&mut self) {
        self.clear_timeout();
        self.targets = None;
    }
    fn clear_timeout(&self) {
        self.timeout.replace(None);
    }
    fn draw_line(
        &self,
        ctx: &CanvasRenderingContext2d,
        src: &Point,
        dst: &Point,
        width: f64,
        color: &String,
    ) {
        ctx.begin_path();
        ctx.set_line_width(width);
        ctx.set_stroke_style_str(&color);
        ctx.move_to(src.x, src.y);
        ctx.line_to(dst.x, dst.y);
        ctx.close_path();
        ctx.stroke();
    }

    pub fn render_node(
        &self,
        node: &Node,
        diagram: &DiagramCore,
        cache: &ImgCache,
        is_node: bool,
    ) -> Result<(), JsValue> {
        match &self.targets {
            Some(targets) => {
                let nodes = match is_node {
                    true => &targets.nodes,
                    false => &targets.boxes,
                };
                self.draw_node(nodes, node, diagram, cache, false)
            }
            None => Ok(()),
        }
    }

    pub fn draw_node(
        &self,
        nodes: &CanvasRenderingContext2d,
        node: &Node,
        diagram: &DiagramCore,
        cache: &ImgCache,
        highlight: bool,
    ) -> Result<(), JsValue> {
        self.draw_box(
            nodes,
            &node.layout,
            &diagram.render_ops,
            diagram.get_opt(node.opt),
            highlight,
            &cache,
        )
    }
    pub fn render_link(
        &self,
        link: &LinkContainer,
        diagram: &DiagramCore,
        opt: &DiagramOpt,
        cache: &ImgCache,
    ) -> Result<(), JsValue> {
        let targets = match &self.targets {
            Some(t) => t,
            None => return Ok(()),
        };
        self.draw_link(
            &targets.links,
            &targets.animations,
            link,
            diagram,
            opt,
            cache,
        )
    }
    pub fn draw_link(
        &self,
        links: &CanvasRenderingContext2d,
        animations: &CanvasRenderingContext2d,
        link: &LinkContainer,
        diagram: &DiagramCore,
        opt: &DiagramOpt,
        cache: &ImgCache,
    ) -> Result<(), JsValue> {
        let data = &link.draw_data;
        let width = data.line_width;
        for (i, ld) in link.ls.links.iter().enumerate() {
            let (a, b) = &data.links[i];
            self.draw_line(links, a, b, width, &diagram.get_opt(ld.opt).color);
        }
        for (a, b, width) in &data.animations {
            self.draw_line(animations, a, b, *width, &opt.animation_color);
        }
        for (i, bundle) in link.ls.bundles.iter().enumerate() {
            let target = data.bundle_draw_box(i);
            self.draw_box(
                links,
                &target,
                opt,
                diagram.get_opt(bundle.opt),
                false,
                &cache,
            )?;
        }
        Ok(())
    }

    pub fn clear(&self) -> Result<(), JsValue> {
        let targets = match &self.targets {
            Some(t) => t,
            None => return Ok(()),
        };
        let t = (unsafe { self.diagram.upgrade().unwrap_unchecked() })
            .borrow()
            .transform;
        let x = t.x;
        let y = t.y;
        let k = t.k;

        for c in [
            &targets.boxes,
            &targets.links,
            &targets.animations,
            &targets.nodes,
            &targets.highlight,
        ] {
            c.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
            c.clear_rect(0.0, 0.0, targets.width as f64, targets.height as f64);
            c.set_transform(k, 0.0, 0.0, k, x, y)?;
        }
        Ok(())
    }
    pub fn render(&self) -> Result<(), JsValue> {
        let targets = match &self.targets {
            Some(t) => t,
            None => return Ok(()),
        };
        let boxes = &targets.boxes;
        let links = &targets.links;
        let animations = &targets.animations;
        let nodes = &targets.nodes;
        let d = unsafe { self.diagram.upgrade().unwrap_unchecked() };
        let diagram = &*d.borrow();

        let cache = &diagram.img_cache;
        let opt = &diagram.render_ops;
        animations.set_line_dash_offset(self.frame_tick);
        let node_vec = &diagram.nodes;
        let link_vec = &diagram.links;
        for target in node_vec {
            match target {
                NodeCanvasTarget::Box((node, _)) => {
                    self.draw_node(boxes, node, diagram, cache, false)?;
                }
                NodeCanvasTarget::Node((node, _)) => {
                    self.draw_node(nodes, node, diagram, cache, false)?;
                }
            }
        }
        for link in link_vec {
            self.draw_link(links, animations, link, diagram, opt, cache)?;
        }
        Ok(())
    }

    pub fn draw_box(
        &self,
        ctx: &CanvasRenderingContext2d,
        target: &Square,
        opt: &DiagramOpt,
        o: &ElementOpt,
        highlight: bool,
        cache: &ImgCache,
    ) -> Result<(), JsValue> {
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
}
