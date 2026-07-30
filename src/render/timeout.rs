use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::Window;
pub struct Timeout {
    _cb: Closure<dyn FnMut()>,
    handle: i32,
    window: Window,
}
impl Timeout {
    pub fn new<F>(window: &Window, f: F, timeout: i32) -> Result<Self, JsValue>
    where
        F: FnMut() + 'static,
    {
        let cb = Closure::wrap(Box::new(f));
        let handle = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            timeout,
        )?;

        Ok(Self {
            _cb: cb,
            handle,
            window: window.clone(),
        })
    }
}

impl Drop for Timeout {
    fn drop(&mut self) {
        self.window.clear_timeout_with_handle(self.handle);
    }
}
