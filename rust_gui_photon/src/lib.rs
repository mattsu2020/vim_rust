use rust_gui_core::backend::{GuiBackend, GuiEvent};
use std::collections::VecDeque;

/// Backend implementation for the QNX Photon GUI system.
///
/// Similar to the other backends, this keeps a record of drawing
/// operations and exposes a queued event interface.
#[derive(Default)]
pub struct PhotonBackend {
    /// Recorded draw operations.
    pub drawn: Vec<String>,
    /// Queue of pending events.
    pub events: VecDeque<GuiEvent>,
}

impl PhotonBackend {
    /// Instantiate an empty backend instance.
    pub fn new() -> Self {
        Self { drawn: Vec::new(), events: VecDeque::new() }
    }

    /// Push an event into the internal queue.
    pub fn push_event(&mut self, ev: GuiEvent) {
        self.events.push_back(ev);
    }
}

impl GuiBackend for PhotonBackend {
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
        let mut backend = PhotonBackend::new();
        backend.draw_text("hi");
        backend.push_event(GuiEvent::Key('q'));
        assert_eq!(backend.drawn, vec!["hi".to_string()]);
        assert_eq!(backend.poll_event(), Some(GuiEvent::Key('q')));
    }
}
