use rust_clipboard;
use rust_drawline::advance_color_col;
use rust_gui_core::backend::{GuiBackend, GuiEvent};
use std::collections::VecDeque;

#[cfg(feature = "gtk")]
use gtk::prelude::*;

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

/// Set the system clipboard text through the GTK backend.
pub fn clipboard_set(text: &str) -> Result<(), ()> {
    rust_clipboard::set_string(text)
}

/// Retrieve text from the system clipboard using the GTK backend.
pub fn clipboard_get() -> Option<String> {
    rust_clipboard::get_string()
}

/// Compute the next color column using the shared helper.
pub fn next_color_col(vcol: i32, color_cols: &mut &[i32]) -> bool {
    advance_color_col(vcol, color_cols)
}

#[cfg(feature = "gtk")]
pub fn gtk_available() -> bool {
    gtk::init().is_ok()
}

#[cfg(not(feature = "gtk"))]
pub fn gtk_available() -> bool {
    false
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

    #[test]
    fn clipboard_roundtrip() {
        clipboard_set("gtk clipboard").unwrap();
        assert_eq!(clipboard_get().as_deref(), Some("gtk clipboard"));
    }

    #[test]
    fn color_column_helper() {
        let cols = vec![3, -1];
        let mut slice: &[i32] = &cols;
        assert!(next_color_col(2, &mut slice));
        assert!(!next_color_col(4, &mut slice));
    }

    #[test]
    fn feature_flag_off_by_default() {
        assert!(!gtk_available());
    }
}
