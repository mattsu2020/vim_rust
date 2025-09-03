use rust_gui_core::backend::{GuiBackend, GuiEvent};
use std::collections::VecDeque;

/// Backend implementation for Motif environments.
/// This skeleton backend mirrors the event loop and drawing
/// behaviour of the original C `gui_motif.c` code at a high level
/// while exposing the same window operations API.
#[derive(Default)]
pub struct MotifBackend {
    /// Records text that has been drawn for inspection in tests.
    pub drawn: Vec<String>,
    /// Queue of pending events to be processed by the core.
    pub events: VecDeque<GuiEvent>,
}

impl MotifBackend {
    /// Create a new, empty backend instance.
    pub fn new() -> Self {
        Self { drawn: Vec::new(), events: VecDeque::new() }
    }

    /// Push an event into the internal queue.  This is primarily
    /// used by tests and examples to simulate user input.
    pub fn push_event(&mut self, ev: GuiEvent) {
        self.events.push_back(ev);
    }
}

impl GuiBackend for MotifBackend {
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
        let mut backend = MotifBackend::new();
        backend.draw_text("hi");
        backend.push_event(GuiEvent::Expose);
        assert_eq!(backend.drawn, vec!["hi".to_string()]);
        assert_eq!(backend.poll_event(), Some(GuiEvent::Expose));
    }
}
