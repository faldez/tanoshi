use dominator::traits::StaticEvent;
use wasm_bindgen::JsCast;

pub use dominator::events::*;

pub struct Error {
    #[allow(dead_code)]
    event: web_sys::ErrorEvent,
}

impl StaticEvent for Error {
    const EVENT_TYPE: &'static str = "error";
    
    #[inline]
    fn unchecked_from_event(event: web_sys::Event) -> Self {
        Self {
            event: event.unchecked_into(),
        }
    }
}

impl Error {
    #[allow(dead_code)]
    #[inline]
    pub fn prevent_default(&self) { self.event.prevent_default(); }

    #[allow(dead_code)]
    #[inline]
    pub fn target(&self) -> Option<web_sys::EventTarget> { self.event.target() }

    #[allow(dead_code)]
    #[inline]
    pub fn dyn_target<A>(&self) -> Option<A> where A: JsCast {
        self.target()?.dyn_into().ok()
    }
}

pub struct PopState {
    event: web_sys::PopStateEvent,
}

impl StaticEvent for PopState {
    const EVENT_TYPE: &'static str = "popstate";

    #[inline]
    fn unchecked_from_event(event: web_sys::Event) -> Self {
        Self {
            event: event.unchecked_into(),
        }
    }
}

impl PopState {
    #[allow(dead_code)]
    #[inline]
    pub fn prevent_default(&self) { self.event.prevent_default(); }

    #[inline]
    pub fn target(&self) -> Option<web_sys::EventTarget> { self.event.target() }

    #[allow(dead_code)]
    #[inline]
    pub fn dyn_target<A>(&self) -> Option<A> where A: JsCast {
        self.target()?.dyn_into().ok()
    }
}