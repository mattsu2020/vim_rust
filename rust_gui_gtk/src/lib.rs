use rust_gui_core::backend::{GuiBackend, GuiEvent};
use std::collections::VecDeque;

/// Backend implementation for GTK environments on Unix.
/// This sample backend records drawing operations and stores events
/// in a queue so it can be easily tested without a full GTK stack.
#[derive(Default)]
pub struct GtkBackend {
    pub drawn: Vec<String>,
    pub events: VecDeque<GuiEvent>,
}

impl GtkBackend {
    pub fn new() -> Self {
        Self {
            drawn: Vec::new(),
            events: VecDeque::new(),
        }
    }

    /// Queue an event for later processing; primarily used in tests.
    pub fn push_event(&mut self, ev: GuiEvent) {
        self.events.push_back(ev);
    }
}

impl GuiBackend for GtkBackend {
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
        let mut backend = GtkBackend::new();
        backend.draw_text("hi");
        backend.push_event(GuiEvent::Key('a'));
        assert_eq!(backend.drawn, vec!["hi".to_string()]);
        assert_eq!(backend.poll_event(), Some(GuiEvent::Key('a')));
    }
}
