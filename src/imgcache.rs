use rustc_hash::FxHashMap;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, HtmlImageElement};

use crate::{ElementOpt, diagram::DiagramCore};

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
    pub diagram: Weak<RefCell<DiagramCore>>,
    pub cache: Rc<RefCell<Cache>>,
}
pub struct Cache {
    pub imgs: FxHashMap<String, CacheState>,
    pub loading: u32,
    pub bulk: bool,
}

impl ImgCache {
    pub fn new(diagram: Weak<RefCell<DiagramCore>>) -> Self {
        Self {
            diagram: diagram,
            cache: Rc::new(RefCell::new(Cache {
                imgs: FxHashMap::default(),
                loading: 0,
                bulk: false,
            })),
        }
    }
    pub fn on_load(&self, src: &String, state: CacheState) {
        {
            let mut cache = self.cache.borrow_mut();
            cache.loading -= 1;
            cache.imgs.insert(src.clone(), state);
            if cache.bulk {
                return;
            }
        }

        unsafe { self.diagram.upgrade().unwrap_unchecked() }
            .borrow()
            .on_img(self);
    }

    pub fn is_done(&self) -> bool {
        self.cache.borrow().loading == 0
    }
    pub fn load_images(&self, opts: &Vec<ElementOpt>) {
        self.cache.borrow_mut().bulk = true;
        for opt in opts {
            self.load_img(&opt.img);
        }

        self.cache.borrow_mut().bulk = false;
        if self.is_done() {
            unsafe { self.diagram.upgrade().unwrap_unchecked() }
                .borrow()
                .on_img(&self);
        }
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

        self.cache.borrow_mut().loading += 1;
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
            diagram: self.diagram.clone(),
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
                cache.on_load(url, CacheState::Failed(e));
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
            // This will drop self
            wanted.on_load(&src, CacheState::Loaded(img_ok.clone()));
        }));
        res.img.set_onload(Some(on_load.as_ref().unchecked_ref()));
        res.onload = Some(on_load);

        let src = url.clone();
        let wanted = cache.clone();
        let on_err = Closure::wrap(Box::new(move |e: ErrorEvent| {
            // This will drop self
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
