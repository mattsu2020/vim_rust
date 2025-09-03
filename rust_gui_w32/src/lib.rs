use rust_gui_core::backend::{GuiBackend, GuiEvent};
use std::collections::VecDeque;
use rust_clipboard;

/// Minimal Windows backend.  Real drawing is delegated to the platform APIs
/// but for now these methods record actions for testing purposes.
#[derive(Default)]
pub struct W32Backend {
    pub drawn: Vec<String>,
    pub events: VecDeque<GuiEvent>,
}

impl W32Backend {
    pub fn new() -> Self {
        Self { drawn: Vec::new(), events: VecDeque::new() }
    }

    /// Queue an event so it can later be retrieved by `poll_event`.
    pub fn push_event(&mut self, ev: GuiEvent) {
        self.events.push_back(ev);
    }
}

impl GuiBackend for W32Backend {
    fn draw_text(&mut self, text: &str) {
        self.drawn.push(text.to_string());
    }

    fn poll_event(&mut self) -> Option<GuiEvent> {
        self.events.pop_front()
    }
}

/// Set the Windows clipboard contents to `text`.
pub fn clipboard_set(text: &str) -> Result<(), ()> {
    rust_clipboard::set_string(text)
}

/// Retrieve text from the Windows clipboard if available.
pub fn clipboard_get() -> Option<String> {
    rust_clipboard::get_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_and_queue_event() {
        let mut backend = W32Backend::new();
        backend.draw_text("hello");
        backend.push_event(GuiEvent::Click { x: 1, y: 2 });
        assert_eq!(backend.drawn, vec!["hello".to_string()]);
        assert_eq!(backend.poll_event(), Some(GuiEvent::Click { x: 1, y: 2 }));
    }

    #[test]
    fn clipboard_roundtrip() {
        clipboard_set("w32 clipboard").unwrap();
        assert_eq!(clipboard_get().as_deref(), Some("w32 clipboard"));
    }
}
