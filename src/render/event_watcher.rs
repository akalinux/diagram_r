use wasm_bindgen::prelude::*;
use web_sys::{AddEventListenerOptions, Event, HtmlElement};
pub struct HtmlEventWatcher {
    target: String,
    cb: Closure<dyn FnMut(Event)>,
    element: HtmlElement,
}

impl HtmlEventWatcher {
    pub fn new<F>(s: &str, f: F, element: &HtmlElement) -> Result<Self, JsValue>
    where
        F: FnMut(Event) + 'static,
    {
        let target = String::from(s);
        let cb = Closure::wrap(Box::new(f));
        let options = AddEventListenerOptions::new();
        options.set_passive(false);
        element.add_event_listener_with_callback_and_add_event_listener_options(
            &target,
            cb.as_ref().unchecked_ref(),
            &options,
        )?;

        Ok(Self {
            element: element.clone(),
            target,
            cb,
        })
    }
    pub fn clear(&self) {
        let _ = self.element.remove_event_listener_with_callback_and_bool(
            &self.target,
            self.cb.as_ref().unchecked_ref(),
            false,
        );
    }
}
impl Drop for HtmlEventWatcher {
    fn drop(&mut self) {
        self.clear();
    }
}
