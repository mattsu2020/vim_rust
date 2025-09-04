pub mod backend;

pub use backend::{GuiBackend, GuiEvent};

/// Runtime state for the generic GUI implementation.
///
/// Only a very small subset of the historic [`gui_T`](../../src/gui.c)
/// structure is modelled here; additional fields can be added as more of the
/// old C implementation is translated.
#[derive(Debug, Clone)]
pub struct GuiState {
    /// Number of character columns in the grid.
    pub num_cols: i32,
    /// Number of character rows in the grid.
    pub num_rows: i32,
    /// Width of a single cell in pixels.
    pub char_width: i32,
    /// Height of a single cell in pixels.
    pub char_height: i32,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            num_cols: 0,
            num_rows: 0,
            char_width: 8,
            char_height: 16,
        }
    }
}

/// Core GUI handling that delegates to a backend implementation.
///
/// This struct integrates logic from the historical `gui.c` and
/// `gui_xmdlg.c` files into a safe Rust API.  Drawing and event
/// processing are performed by a backend implementation which is
/// selected per operating system at compile time.
#[derive(Default)]
pub struct GuiCore<B: GuiBackend> {
    backend: B,
    state: GuiState,
}

impl<B: GuiBackend> GuiCore<B> {
    /// Create a new GUI core with the given backend and default state.
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            state: GuiState::default(),
        }
    }

    /// Create a new GUI core with an explicit [`GuiState`].
    pub fn with_state(backend: B, state: GuiState) -> Self {
        Self { backend, state }
    }

    /// Access the backend mutably.  Mainly used for tests to
    /// inspect the internal state.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Read-only access to the GUI state.
    pub fn state(&self) -> &GuiState {
        &self.state
    }

    /// Update the known grid size.
    pub fn resize(&mut self, cols: i32, rows: i32) {
        self.state.num_cols = cols;
        self.state.num_rows = rows;
    }

    /// Translate pixel coordinates to a grid position.
    ///
    /// This mirrors the behaviour of the `gui_xy2colrow()` helper in the old
    /// C implementation where the column is returned via pointer and the row
    /// as the function result.  Here we simply return a tuple.
    pub fn xy_to_col_row(&self, x: i32, y: i32) -> (i32, i32) {
        let col = x / self.state.char_width;
        let row = y / self.state.char_height;
        (col, row)
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
            Self {
                drawn: Vec::new(),
                events: VecDeque::new(),
            }
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
        backend.push_event(GuiEvent::Expose);
        let mut core = GuiCore::new(backend);
        core.draw_text("hello");
        let mut seen = Vec::new();
        core.process_events(|e| seen.push(e));
        assert_eq!(core.backend_mut().drawn, vec!["hello".to_string()]);
        assert_eq!(
            seen,
            vec![
                GuiEvent::Key('x'),
                GuiEvent::Click { x: 10, y: 20 },
                GuiEvent::Expose,
            ]
        );
    }

    /// Converting pixel coordinates to grid positions mirrors the behaviour of
    /// `gui_xy2colrow()` from the original C implementation.
    #[test]
    fn xy_to_col_row_basic() {
        let backend = TestBackend::new();
        let core = GuiCore::new(backend);
        // With the default character size of 8x16 a point at (16,32) lies on
        // the third column and third row (0-indexed).
        assert_eq!(core.xy_to_col_row(16, 32), (2, 2));
    }
}
