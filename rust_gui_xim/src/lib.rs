use rust_gui_core::backend::{GuiBackend, GuiEvent};
use std::collections::VecDeque;

#[cfg(have_xim)]
mod ffi {
    use std::ffi::c_void;
    extern "C" {
        pub fn XOpenIM(
            display: *mut c_void,
            rdb: *mut c_void,
            res_class: *mut c_void,
            res_name: *mut c_void,
        ) -> *mut c_void;
    }
}

/// Backend placeholder for X Input Method handling.
pub struct XimBackend {
    events: VecDeque<GuiEvent>,
}

impl XimBackend {
    pub fn new() -> Self {
        Self { events: VecDeque::new() }
    }

    pub fn push_event(&mut self, ev: GuiEvent) {
        self.events.push_back(ev);
    }

    #[cfg(have_xim)]
    pub fn has_xim() -> bool {
        true
    }

    #[cfg(not(have_xim))]
    pub fn has_xim() -> bool {
        false
    }
}

impl GuiBackend for XimBackend {
    fn draw_text(&mut self, _text: &str) {}

    fn poll_event(&mut self) -> Option<GuiEvent> {
        self.events.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue() {
        let mut xim = XimBackend::new();
        xim.push_event(GuiEvent::Expose);
        assert_eq!(xim.poll_event(), Some(GuiEvent::Expose));
    }

    #[test]
    fn config_flag() {
        if cfg!(have_xim) {
            assert!(XimBackend::has_xim());
        } else {
            assert!(!XimBackend::has_xim());
        }
    }
}
