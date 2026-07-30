use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, ResizeObserver, ResizeObserverEntry};

pub struct SizeWatcher {
    _callback: Closure<dyn FnMut(Vec<ResizeObserverEntry>, ResizeObserver)>,
    watcher: ResizeObserver,
}

impl SizeWatcher {
    pub fn new<F>(el: &HtmlElement, f: F) -> Result<Self, JsValue>
    where
        F: FnMut(Vec<ResizeObserverEntry>, ResizeObserver) + 'static,
    {
        let cb = Closure::wrap(Box::new(f));
        match ResizeObserver::new(cb.as_ref().unchecked_ref()) {
            Ok(o) => {
                o.observe(el);
                Ok(Self {
                    watcher: o,
                    _callback: cb,
                })
            }
            Err(e) => Err(e),
        }
    }
}

impl Drop for SizeWatcher {
    fn drop(&mut self) {
        self.watcher.disconnect();
    }
}
