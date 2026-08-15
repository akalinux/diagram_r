use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
pub mod canvasrender;
pub mod event_watcher;
pub mod size_watcher;
pub mod timeout;
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;
pub mod pointerwatcher;
use crate::{
    Point,
    bsp::ScreenSlot,
    diagram::{DiagramCore, LinkAndElement},
};

pub trait BuildRender {
    fn new(
        canvas: &HtmlCanvasElement,
        diagram: Weak<RefCell<DiagramCore>>,
    ) -> Result<Box<dyn CoreRender>, JsValue>;
}

pub trait CoreRender {
    fn render(&self) -> Result<(), JsValue>;
    fn update(&self, target: ScreenSlot, distance: &Point);
    fn clear(&self);
}

#[derive(Debug, PartialEq, Eq)]
pub struct HighlightTargets {
    pub nodes: Vec<usize>,
    pub boxes: Vec<usize>,
    pub links: Vec<LinkAndElement>,
    pub bundles: Vec<LinkAndElement>,
}

/*
pub struct Render {
    pub diagram: Weak<RefCell<DiagramCore>>,
    this: Weak<RefCell<Self>>,
    targets: Option<Targets>,
    timeout: RefCell<Option<Timeout>>,
    frame_tick: f64,
    current_target: RefCell<Option<(LookupPointResult, Point)>>,
    highlights: RefCell<Option<HighlightTargets>>,
}
impl Render {
    pub fn new() -> Rc<RefCell<Self>> {
        let res = Self {
            diagram: Weak::new(),
            this: Weak::new(),
            highlights: RefCell::new(None),
            targets: None,
            frame_tick: 0.0,
            timeout: RefCell::new(None),
            current_target: RefCell::new(None),
        };
        let rc = Rc::new(RefCell::new(res));
        rc.borrow_mut().this = Rc::downgrade(&rc);

        rc
    }
    fn set_timeout(&self) {
        let this = unsafe { self.this.upgrade().unwrap_unchecked() };

        let job = move || {
            match this.borrow().current_target.replace(None) {
                Some((l, p)) => match l {
                    LookupPointResult::NoMatch => {
                        let d = unsafe { this.borrow().diagram.upgrade().unwrap_unchecked() };
                        let res = d.borrow().contains_point(&this.borrow().to_map_xy(&p));
                        d.borrow().run_callback(UiEvent::MouseOver(res), &p);
                    }
                    _ => (),
                },
                None => (),
            };
        };
        match Timeout::new(
            job,
            unsafe { self.diagram.upgrade().unwrap_unchecked() }
                .borrow()
                .render_ops
                .timeout,
        ) {
            Err(_) => return,
            Ok(t) => {
                self.timeout.replace(Some(t));
            }
        };
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
        self.set_timeout();
    }
    pub fn on_mouse_enter(&self, p: &Point) {
        self.current_target
            .replace(Some((LookupPointResult::NoMatch, *p)));
    }
    pub fn on_mouse_leave(&self, _: &Point) {
        self.clear_timeout();
        self.current_target.replace(None);
        self.highlights.replace(None);
    }
    pub fn on_mouse_move(&self, p: &Point) {
        self.clear_timeout();
        self.highlights.replace(None);
        match self.current_target.borrow_mut().as_mut() {
            Some((l, op)) => {
                let nodes = match l {
                    LookupPointResult::NoMatch => {
                        *op = *p;
                        self.set_timeout();
                        return;
                    }
                    LookupPointResult::Screen => {
                        let distance = self.to_map_xy(op).get_move_distance(p);
                        *op = *p;
                        let mut t = unsafe { self.diagram.upgrade().unwrap_unchecked() }
                            .borrow()
                            .get_transform();

                        t.x += distance.x;
                        t.y += distance.y;

                        unsafe { self.diagram.upgrade().unwrap_unchecked() }
                            .borrow_mut()
                            .set_transform(t);
                        let _ = self.render();
                        return;
                    }
                    LookupPointResult::Box(id) => {
                        unsafe { self.diagram.upgrade().unwrap_unchecked() }
                            .borrow()
                            .get_related_nodes(&[GroupID::Box(*id)])
                    }
                    LookupPointResult::Node(id) => {
                        unsafe { self.diagram.upgrade().unwrap_unchecked() }
                            .borrow()
                            .get_related_nodes(&[GroupID::Node(*id)])
                    }
                    LookupPointResult::Bundle((link_id, _))
                    | LookupPointResult::Link((link_id, _)) => {
                        let r = self.diagram.upgrade();
                        let d = unsafe { r.unwrap_unchecked() };

                        let link = &d.borrow().links[*link_id].ls;
                        vec![GroupID::Node(link.src), GroupID::Node(link.dst)]
                    }

                    _ => return,
                };
                let distance = self.to_map_xy(op).get_move_distance(p);
                unsafe { self.diagram.upgrade().unwrap_unchecked() }
                    .borrow_mut()
                    .move_nodes(&distance, &nodes);
                *op = *p;
                let _ = self.render();
                return;
            }
            None => (),
        }
        // if we got here.. then we need to setup for timeout
        self.current_target
            .replace(Some((LookupPointResult::NoMatch, *p)));
        self.set_timeout();
    }
    pub fn on_mouse_wheel(&self, p: &Point, delta: f64) {}
    pub fn to_map_xy(&self, p: &Point) -> Point {
        let t = unsafe { self.diagram.upgrade().unwrap_unchecked() }
            .borrow()
            .get_transform();
        to_map_xy(p, &t)
    }
    pub fn mount(
        &mut self,
        width: u32,
        height: u32,
        canvas: HtmlCanvasElement,
    ) -> Result<(), JsValue> {
        let targets = Targets::new(
            width,
            height,
            unsafe { self.this.upgrade().unwrap_unchecked() },
            canvas,
        )?;

        let d = unsafe { self.diagram.upgrade().unwrap_unchecked() };
        let diagram = d.borrow();
        let ops = &diagram.render_ops;
        targets.context.set_font(&ops.font_family);

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
    ) -> Result<(), JsValue> {
        match &self.targets {
            Some(targets) => self.draw_node(&targets.context, node, diagram, cache, false),
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
        self.draw_link(&targets.context, link, diagram, opt, cache)
    }
    pub fn draw_link(
        &self,
        canvas: &CanvasRenderingContext2d,
        link: &LinkContainer,
        diagram: &DiagramCore,
        opt: &DiagramOpt,
        cache: &ImgCache,
    ) -> Result<(), JsValue> {
        let data = &link.draw_data;
        let width = data.line_width;
        for (i, ld) in link.ls.links.iter().enumerate() {
            let (a, b) = &data.links[i];
            self.draw_line(canvas, a, b, width, &diagram.get_opt(ld.opt).color);
        }

        canvas.set_line_dash(&opt.animation_dash())?;
        for (a, b, width) in &data.animations {
            self.draw_line(canvas, a, b, *width, &opt.animation_color);
        }
        canvas.set_line_dash(&Array::new())?;

        for (i, bundle) in link.ls.bundles.iter().enumerate() {
            let target = data.bundle_draw_box(i);
            self.draw_box(
                canvas,
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
        let c = &targets.context;

        c.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
        c.clear_rect(0.0, 0.0, targets.width as f64, targets.height as f64);
        c.set_transform(k, 0.0, 0.0, k, x, y)?;
        Ok(())
    }

    pub fn render(&self) -> Result<(), JsValue> {
        let targets = match &self.targets {
            Some(t) => t,
            None => return Ok(()),
        };
        let context = &targets.context;
        context.set_global_alpha(1.0);
        context.set_line_dash_offset(self.frame_tick);
        let d = unsafe { self.diagram.upgrade().unwrap_unchecked() };
        let diagram = &*d.borrow();

        let cache = &diagram.img_cache;
        let opt = &diagram.render_ops;
        let node_vec = &diagram.nodes;
        let boxes_vec = &diagram.boxes;
        let link_vec = &diagram.links;
        for node in boxes_vec {
            self.draw_node(context, node, diagram, cache, false)?;
        }

        for link in link_vec {
            self.draw_link(context, link, diagram, opt, cache)?;
        }
        for (node, _) in node_vec {
            self.draw_node(context, node, diagram, cache, false)?;
        }

        let h = self.highlights.borrow();
        let highlights = match h.as_ref() {
            Some(h) => h,
            // # -- STOP HERE IF WE HAVE NOTHING TO HIGHLIGHT! -- #
            None => return Ok(()),
        };

        let highight_color = &opt.highlight_color;
        context.set_global_alpha(opt.highlight_alpha);
        for id in &highlights.boxes {
            let node = &boxes_vec[*id];
            self.draw_box(
                context,
                &node.layout,
                opt,
                diagram.get_opt(node.opt),
                true,
                cache,
            )?;
        }

        for set in &highlights.links {
            let link = &link_vec[set.link];
            let src = node_vec[link.ls.src].0.layout.get_center();
            let dst = node_vec[link.ls.dst].0.layout.get_center();
            let width = link.draw_data.line_width * opt.highlight_scale;

            self.draw_line(context, &src, &dst, width, highight_color);
        }
        for set in &highlights.bundles {
            let link = &link_vec[set.link];
            let bundle = &link.ls.bundles[set.element];
            let target = link.draw_data.bundle_draw_box(set.element);
            self.draw_box(
                context,
                &target,
                opt,
                diagram.get_opt(bundle.opt),
                true,
                cache,
            )?;
        }
        for id in &highlights.nodes {
            let node = &node_vec[*id].0;
            self.draw_box(
                context,
                &node.layout,
                opt,
                diagram.get_opt(node.opt),
                true,
                cache,
            )?;
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
*/
