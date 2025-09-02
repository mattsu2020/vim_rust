use super::{GuiBackend, GuiEvent};
use std::collections::VecDeque;

/// Backend implementation for GTK environments on Unix.
/// Drawing operations are recorded and events are stored in a queue.
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
