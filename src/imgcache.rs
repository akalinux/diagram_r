use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, HtmlImageElement};

use crate::render::Render;

pub struct ImgLoader {
    onload: Option<Closure<dyn FnMut()>>,
    onerr: Option<Closure<dyn FnMut(ErrorEvent)>>,
    pub img: HtmlImageElement,
}

pub enum CacheState {
    Downloading(Rc<RefCell<ImgLoader>>),
    Loaded(HtmlImageElement),
    Failed(JsValue),
}

pub struct ImgCache {
    pub cache: HashMap<String, CacheState>,
    render: Rc<RefCell<Render>>,
    loading: u32,
    this: Weak<RefCell<ImgCache>>,
}

impl ImgCache {
    pub fn new(render: Rc<RefCell<Render>>) -> Rc<RefCell<Self>> {
        let res = Self {
            render,
            loading: 0,
            cache: HashMap::new(),
            this: Weak::new(),
        };

        let this = Rc::new(RefCell::new(res));
        let dg = Rc::downgrade(&this);
        this.borrow_mut().this = dg;

        this
    }
    pub fn on_load(&mut self, src: &String, state: CacheState) {
        self.loading -= 1;
        self.cache.insert(src.clone(), state);
        self.render.borrow_mut().on_img(self);
    }
    pub fn uptick(&mut self) {
        self.loading += 1;
    }

    pub fn load_img(&mut self, url: &String) -> Option<Result<HtmlImageElement, JsValue>> {
        if url.is_empty() {
            return None;
        }
        match self.cache.get(url) {
            Some(cs) => match cs {
                CacheState::Downloading(_) => return None,
                CacheState::Loaded(img) => return Some(Ok(img.clone())),
                CacheState::Failed(e) => return Some(Err(e.clone())),
            },
            _ => (),
        }

        match ImgLoader::new(url, unsafe { self.this.upgrade().unwrap_unchecked() }) {
            Ok(_) => (),
            Err(msg) => return Some(Err(msg)),
        };

        match self.cache.get(url) {
            Some(cs) => match cs {
                CacheState::Downloading(_) => None,
                CacheState::Loaded(img) => Some(Ok(img.clone())),
                CacheState::Failed(e) => Some(Err(e.clone())),
            },
            _ => None,
        }
    }
}

impl ImgLoader {
    pub fn new(url: &String, cache: Rc<RefCell<ImgCache>>) -> Result<(), JsValue> {
        let img = HtmlImageElement::new()?;

        let mut res = Self {
            onerr: None,
            onload: None,
            img,
        };

        let img_ok = res.img.clone();
        let src = url.clone();

        let wanted = cache.clone();
        let on_load = Closure::wrap(Box::new(move || {
            // This call causes self to drop
            wanted
                .borrow_mut()
                .on_load(&src, CacheState::Loaded(img_ok.clone()));
        }));
        res.img.set_onload(Some(on_load.as_ref().unchecked_ref()));
        res.onload = Some(on_load);

        let src = url.clone();
        let wanted = cache.clone();
        let on_err = Closure::wrap(Box::new(move |e: ErrorEvent| {
            // This call causes self to drop
            wanted
                .borrow_mut()
                .on_load(&src, CacheState::Failed(e.into()));
        }));
        res.img.set_onerror(Some(on_err.as_ref().unchecked_ref()));
        res.onerr = Some(on_err);

        let cp = res.img.clone();
        //let cache = cache.clone();
        cache.borrow_mut().cache.insert(
            url.clone(),
            CacheState::Downloading(Rc::new(RefCell::new(res))),
        );

        // this can run the callback before we return a value!
        cp.set_src(url);

        return Ok(());
    }
    pub fn clear(&mut self) {
        self.img.set_onload(None);
        self.img.set_onerror(None);
        self.onerr = None;
        self.onload = None;
    }
}
impl Drop for ImgLoader {
    fn drop(&mut self) {
        // prevent circular refs!
        self.clear()
    }
}
