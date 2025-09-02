use rust_gui_core::backend::{GuiBackend, GuiEvent};
use rust_gui_core::GuiCore;
use std::collections::VecDeque;

#[derive(Default)]
pub struct MotifBackend {
    pub drawn: Vec<String>,
    pub events: VecDeque<GuiEvent>,
}

impl MotifBackend {
    pub fn new() -> Self {
        Self { drawn: Vec::new(), events: VecDeque::new() }
    }

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

#[no_mangle]
pub extern "C" fn rs_gui_motif_event_loop() {
    let backend = MotifBackend::new();
    let mut gui = GuiCore::new(backend);
    gui.draw_text("Vim Rust Motif GUI");
    while let Some(ev) = gui.backend_mut().poll_event() {
        if let GuiEvent::Expose = ev {
            gui.draw_text("Vim Rust Motif GUI");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_and_draw() {
        let mut backend = MotifBackend::new();
        backend.draw_text("hi");
        backend.push_event(GuiEvent::Key('a'));
        assert_eq!(backend.drawn, vec!["hi".to_string()]);
        assert_eq!(backend.poll_event(), Some(GuiEvent::Key('a')));
    }
}
