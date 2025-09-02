use super::{GuiBackend, GuiEvent};

/// Minimal Windows backend.  Real drawing is delegated to the platform APIs
/// but for now these methods are no-ops.
#[derive(Default)]
pub struct W32Backend;

impl W32Backend {
    pub fn new() -> Self {
        Self
    }
}

impl GuiBackend for W32Backend {
    fn draw_text(&mut self, _text: &str) {}
    fn poll_event(&mut self) -> Option<GuiEvent> {
        None
    }
}
