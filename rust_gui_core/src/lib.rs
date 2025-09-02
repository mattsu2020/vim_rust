pub mod backend;

pub use backend::{GuiBackend, GuiEvent};

/// Core GUI handling that delegates to a backend implementation.
///
/// This struct integrates logic from the historical `gui.c` and
/// `gui_xmdlg.c` files into a safe Rust API.  Drawing and event
/// processing are performed by a backend implementation which is
/// selected per operating system at compile time.
#[derive(Default)]
pub struct GuiCore<B: GuiBackend> {
    backend: B,
}

impl<B: GuiBackend> GuiCore<B> {
    /// Create a new GUI core with the given backend.
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Access the backend mutably.  Mainly used for tests to
    /// inspect the internal state.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Draw text using the backend implementation.
    pub fn draw_text(&mut self, text: &str) {
        self.backend.draw_text(text);
    }

    /// Process all pending events, calling `handler` for each one.
    pub fn process_events<F: FnMut(GuiEvent)>(&mut self, mut handler: F) {
        while let Some(event) = self.backend.poll_event() {
            handler(event);
        }
    }
}

/// Simple dialog helpers used by the old GUI code.
pub mod dialog {
    /// Display a message to the user.  For now this simply prints to stdout
    /// which keeps the implementation portable and safe for tests.
    pub fn message(title: &str, text: &str) {
        println!("[{title}] {text}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::VecDeque;

    #[derive(Default)]
    struct TestBackend {
        drawn: Vec<String>,
        events: VecDeque<GuiEvent>,
    }

    impl TestBackend {
        fn new() -> Self {
            Self { drawn: Vec::new(), events: VecDeque::new() }
        }
        fn push_event(&mut self, ev: GuiEvent) {
            self.events.push_back(ev);
        }
    }

    impl GuiBackend for TestBackend {
        fn draw_text(&mut self, text: &str) {
            self.drawn.push(text.to_string());
        }

        fn poll_event(&mut self) -> Option<GuiEvent> {
            self.events.pop_front()
        }
    }

    /// Verify that drawing forwards to the backend and that queued
    /// events are processed in order.
    #[test]
    fn draw_and_process_events() {
        let mut backend = TestBackend::new();
        backend.push_event(GuiEvent::Key('x'));
        backend.push_event(GuiEvent::Click { x: 10, y: 20 });
        let mut core = GuiCore::new(backend);
        core.draw_text("hello");
        let mut seen = Vec::new();
        core.process_events(|e| seen.push(e));
        assert_eq!(core.backend_mut().drawn, vec!["hello".to_string()]);
        assert_eq!(
            seen,
            vec![GuiEvent::Key('x'), GuiEvent::Click { x: 10, y: 20 }]
        );
    }
}
