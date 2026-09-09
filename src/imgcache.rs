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
    Downloading(Box<(String, ImgLoader)>),
    Loaded(Box<(String, HtmlImageElement)>),
    Failed(Box<(String, JsValue)>),
    NeverLoaded,
}
impl Default for CacheState {
    fn default() -> Self {
        CacheState::NeverLoaded
    }
}

pub struct ImgCache {
    pub diagram: Weak<RefCell<DiagramCore>>,
    pub cache: Rc<RefCell<Cache>>,
}
pub struct Cache {
    pub imgs: Box<[CacheState]>,
    pub loading: u32,
    pub bulk: bool,
}

impl ImgCache {
    pub fn new(diagram: Weak<RefCell<DiagramCore>>) -> Self {
        Self {
            diagram: diagram,
            cache: Rc::new(RefCell::new(Cache {
                imgs: Box::new([]),
                loading: 0,
                bulk: false,
            })),
        }
    }
    pub fn on_load(&self, src: usize, state: CacheState) {
        {
            let mut cache = self.cache.borrow_mut();
            cache.loading -= 1;
            cache.imgs[src] = state;
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
    pub fn load_images(&self, opts: &Box<[ElementOpt]>) {
        self.cache.borrow_mut().bulk = true;
        let mut imgs = Vec::with_capacity(opts.len());
        imgs.resize_with(opts.len(), CacheState::default);
        self.cache.borrow_mut().imgs = imgs.into_boxed_slice();
        for opt in opts.iter() {
            self.load_img(opt);
        }

        self.cache.borrow_mut().bulk = false;
        if self.is_done() {
            unsafe { self.diagram.upgrade().unwrap_unchecked() }
                .borrow()
                .on_img(&self);
        }
    }
    pub fn get_img(&self, id: usize) -> Option<HtmlImageElement> {
        match self.cache.borrow().imgs.get(id) {
            Some(cs) => match cs {
                CacheState::Loaded(data) => Some(data.1.clone()),
                _ => None,
            },
            _ => None,
        }
    }
    pub fn update(&self, opt: &ElementOpt) {
        let add = {
            let cs = self.cache.borrow();
            let slot = match cs.imgs.get(opt.id) {
                Some(s) => s,
                None => return,
            };
            let (cmp, add) = match slot {
                CacheState::Downloading(data) => (&data.0, 0),
                CacheState::Loaded(data) => (&data.0, 1),
                CacheState::Failed(data) => (&data.0, 1),
                // this should not be possible!
                _ => return,
            };
            if *cmp == opt.img {
                return;
            }
            add
        };
        {
            let mut cache = self.cache.borrow_mut();
            cache.imgs[opt.id] = CacheState::NeverLoaded;
            cache.loading += add;
        }
        ImgLoader::new(opt, self.clone());
    }
    fn load_img(&self, opt: &ElementOpt) {
        match self.cache.borrow().imgs.get(opt.id) {
            Some(cs) => match cs {
                CacheState::Downloading(_) | CacheState::Failed(_) | CacheState::Loaded(_) => {
                    return;
                }
                CacheState::NeverLoaded => (),
            },
            _ => (),
        }

        self.cache.borrow_mut().loading += 1;
        ImgLoader::new(opt, self.clone());
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
    pub fn new(opt: &ElementOpt, cache: ImgCache) {
        let img = match HtmlImageElement::new() {
            Ok(i) => i,
            Err(e) => {
                cache.on_load(opt.id, CacheState::Failed(Box::new((opt.img.clone(), e))));
                return;
            }
        };

        let mut res = Self {
            onerr: None,
            onload: None,
            img,
        };

        let img_ok = res.img.clone();

        let wanted = cache.clone();
        let src = opt.img.clone();
        let id = opt.id;
        let on_load = Closure::wrap(Box::new(move || 
            // This will drop self
            wanted.on_load(
                id,
                CacheState::Loaded(Box::new((src.clone(), img_ok.clone()))),
            )
        ));
        res.img.set_onload(Some(on_load.as_ref().unchecked_ref()));
        res.onload = Some(on_load);

        let src = opt.img.clone();
        let wanted = cache.clone();
        let on_err = Closure::wrap(Box::new(move |e: ErrorEvent| 
            // This will drop self
            wanted.on_load(id, CacheState::Failed(Box::new((src.clone(), e.into()))))
        ));
        res.img.set_onerror(Some(on_err.as_ref().unchecked_ref()));
        res.onerr = Some(on_err);
        let img = res.img.clone();

        cache.cache.borrow_mut().imgs[id] =
            CacheState::Downloading(Box::new((opt.img.clone(), res)));

        // this can run the callback before a value can be returned..
        // So there is no point in having a return statement!
        img.set_src(&opt.img);
    }
}
impl Drop for ImgLoader {
    fn drop(&mut self) {
        // prevent circular refs!
        self.img.set_onload(None);
        self.img.set_onerror(None);
    }
}
