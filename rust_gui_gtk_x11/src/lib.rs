use rust_gui_core::backend::{GuiBackend, GuiEvent};
use std::collections::VecDeque;

/// Backend implementation representing the hybrid GTK/X11 layer.
/// This example backend keeps compatibility with the existing
/// window APIs by recording draw operations and exposing a simple
/// queued event loop.
#[derive(Default)]
pub struct GtkX11Backend {
    /// Recorded drawing operations for test inspection.
    pub drawn: Vec<String>,
    /// Pending events waiting to be processed by the core.
    pub events: VecDeque<GuiEvent>,
}

impl GtkX11Backend {
    /// Construct a fresh backend instance.
    pub fn new() -> Self {
        Self { drawn: Vec::new(), events: VecDeque::new() }
    }

    /// Push an event onto the internal queue.
    pub fn push_event(&mut self, ev: GuiEvent) {
        self.events.push_back(ev);
    }
}

impl GuiBackend for GtkX11Backend {
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
        let mut backend = GtkX11Backend::new();
        backend.draw_text("hi");
        backend.push_event(GuiEvent::Click { x: 1, y: 2 });
        assert_eq!(backend.drawn, vec!["hi".to_string()]);
        assert_eq!(backend.poll_event(), Some(GuiEvent::Click { x: 1, y: 2 }));
    }
}
