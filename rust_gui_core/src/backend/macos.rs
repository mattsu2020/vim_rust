use super::{GuiBackend, GuiEvent};

/// Stub backend for macOS.  A full implementation would bridge to Cocoa APIs.
#[derive(Default)]
pub struct MacBackend;

impl MacBackend {
    pub fn new() -> Self {
        Self
    }
}

impl GuiBackend for MacBackend {
    fn draw_text(&mut self, _text: &str) {}
    fn poll_event(&mut self) -> Option<GuiEvent> {
        None
    }
}
