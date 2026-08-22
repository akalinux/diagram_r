use std::cell::RefCell;
use std::rc::{Rc, Weak};
use wasm_bindgen::prelude::*;
use web_sys::Window;

use crate::constants::WINDOW_ERROR;
use crate::diagram::DiagramCore;

pub struct FrameTimer {
    window: Window,
    ts: f64,
    diagram: Weak<RefCell<DiagramCore>>,
    cb: Option<Closure<dyn FnMut(f64)>>,
    id: Option<i32>,
    timeout: f64,
}

impl FrameTimer {
    pub fn new(diagram: Weak<RefCell<DiagramCore>>) -> Result<Rc<RefCell<FrameTimer>>, JsValue> {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return Err(JsValue::from_str(WINDOW_ERROR)),
        };
        let timeout = (1000
            / unsafe {
                diagram
                    .upgrade()
                    .unwrap_unchecked()
                    .borrow()
                    .render_ops
                    .frame_rate
            }) as f64;

        let res = Self {
            cb: None,
            ts: 0.0,
            window,
            diagram,
            id: None,
            timeout,
        };
        let dst = Rc::new(RefCell::new(res));
        let fc = dst.clone();

        let cb = Closure::new(move |timestamp: f64| fc.borrow_mut().run(timestamp));

        dst.borrow_mut().cb = Some(cb);
        dst.borrow_mut().request_frame()?;

        Ok(dst)
    }
    fn run(&mut self, timestamp: f64) {
        if self.ts <= timestamp {
            let _ = unsafe { self.diagram.upgrade().unwrap_unchecked() }
                .borrow()
                .animate();
            self.ts = timestamp + self.timeout;
        }
        let _ = self.request_frame();
    }
    fn request_frame(&mut self) -> Result<(), JsValue> {
        let cb = unsafe { self.cb.as_mut().unwrap_unchecked() };
        let id = self
            .window
            .request_animation_frame(cb.as_ref().unchecked_ref())?;
        self.id = Some(id);
        Ok(())
    }
}

impl Drop for FrameTimer {
    fn drop(&mut self) {
        match self.id {
            Some(id) => {
                let _ = self.window.cancel_animation_frame(id);
                ()
            }
            _ => (),
        }
    }
}
