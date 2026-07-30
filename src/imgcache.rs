use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, HtmlImageElement};

use crate::render::Render;

pub struct ImgLoader {
    onload: Option<Closure<dyn FnMut()>>,
    onerr: Option<Closure<dyn FnMut(ErrorEvent)>>,
    pub img: HtmlImageElement,
}

pub enum CacheState {
    Downloading(ImgLoader),
    Loaded(HtmlImageElement),
    Failed(JsValue),
}

pub struct ImgCache {
    render: Rc<RefCell<Render>>,
    pub cache: Rc<RefCell<Cache>>,
}
pub struct Cache {
    pub imgs: HashMap<String, CacheState>,
    loading: u32,
}

impl ImgCache {
    pub fn new(render: Rc<RefCell<Render>>) -> Self {
        Self {
            render,
            cache: Rc::new(RefCell::new(Cache {
                imgs: HashMap::new(),
                loading: 0,
            })),
        }
    }
    pub fn on_load(&self, src: &String, state: CacheState) {
        {
            let mut cache = self.cache.borrow_mut();
            cache.loading -= 1;
            cache.imgs.insert(src.clone(), state);
        }
        self.render.borrow().on_img(self);
    }
    pub fn uptick(&mut self) {
        self.cache.borrow_mut().loading += 1;
    }

    pub fn is_done(&self) -> bool {
        self.cache.borrow().loading == 0
    }
    pub fn load_img(&self, url: &String) -> Option<Result<HtmlImageElement, JsValue>> {
        if url.is_empty() {
            return None;
        }
        match self.cache.borrow().imgs.get(url) {
            Some(cs) => match cs {
                CacheState::Downloading(_) => return None,
                CacheState::Loaded(img) => return Some(Ok(img.clone())),
                CacheState::Failed(e) => return Some(Err(e.clone())),
            },
            _ => (),
        }

        ImgLoader::new(url, self.clone());

        match self.cache.borrow().imgs.get(url) {
            Some(cs) => match cs {
                CacheState::Downloading(_) => None,
                CacheState::Loaded(img) => Some(Ok(img.clone())),
                CacheState::Failed(e) => Some(Err(e.clone())),
            },
            _ => None,
        }
    }
}
impl Clone for ImgCache {
    fn clone(&self) -> Self {
        Self {
            render: Rc::clone(&self.render),
            cache: Rc::clone(&self.cache),
        }
    }
}

impl ImgLoader {
    pub fn new(url: &String, cache: ImgCache) {
        let img;
        match HtmlImageElement::new() {
            Ok(i) => img = i,
            Err(e) => {
                cache
                    .cache
                    .borrow_mut()
                    .imgs
                    .insert(url.clone(), CacheState::Failed(e));
                return;
            }
        };

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
            wanted.on_load(&src, CacheState::Loaded(img_ok.clone()));
        }));
        res.img.set_onload(Some(on_load.as_ref().unchecked_ref()));
        res.onload = Some(on_load);

        let src = url.clone();
        let wanted = cache.clone();
        let on_err = Closure::wrap(Box::new(move |e: ErrorEvent| {
            // This call causes self to drop
            wanted.on_load(&src, CacheState::Failed(e.into()));
        }));
        res.img.set_onerror(Some(on_err.as_ref().unchecked_ref()));
        res.onerr = Some(on_err);

        let cp = res.img.clone();
        //let cache = cache.clone();
        cache
            .cache
            .borrow_mut()
            .imgs
            .insert(url.clone(), CacheState::Downloading(res));

        // this can run the callback before we return a value!
        cp.set_src(url);
    }
}
impl Drop for ImgLoader {
    fn drop(&mut self) {
        // prevent circular refs!
        self.img.set_onload(None);
        self.img.set_onerror(None);
    }
}
