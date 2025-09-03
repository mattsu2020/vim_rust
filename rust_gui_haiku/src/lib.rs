use rust_gui_core::backend::{GuiBackend, GuiEvent};
use std::collections::VecDeque;

/// Backend implementation for the Haiku windowing system.
///
/// This mirrors the structure of other GUI backends by recording
/// drawing operations and providing a simple queued event model.
#[derive(Default)]
pub struct HaikuBackend {
    /// Text that has been drawn, stored for inspection in tests.
    pub drawn: Vec<String>,
    /// Pending GUI events.
    pub events: VecDeque<GuiEvent>,
}

impl HaikuBackend {
    /// Create a new backend instance with empty state.
    pub fn new() -> Self {
        Self { drawn: Vec::new(), events: VecDeque::new() }
    }

    /// Queue an event for later processing.
    pub fn push_event(&mut self, ev: GuiEvent) {
        self.events.push_back(ev);
    }
}

impl GuiBackend for HaikuBackend {
    fn draw_text(&mut self, text: &str) {
        self.drawn.push(text.to_string());
    }

    fn poll_event(&mut self) -> Option<GuiEvent> {
        self.events.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_and_draw() {
        let mut backend = HaikuBackend::new();
        backend.draw_text("hi");
        backend.push_event(GuiEvent::Expose);
        assert_eq!(backend.drawn, vec!["hi".to_string()]);
        assert_eq!(backend.poll_event(), Some(GuiEvent::Expose));
    }
}
