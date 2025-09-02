use rust_gui_core::backend::{GuiBackend, GuiEvent};

/// Minimal Windows backend.  Real drawing is delegated to the platform APIs
/// but for now these methods record actions for testing purposes.
#[derive(Default)]
pub struct W32Backend {
    pub drawn: Vec<String>,
}

impl W32Backend {
    pub fn new() -> Self {
        Self { drawn: Vec::new() }
    }
}

impl GuiBackend for W32Backend {
    fn draw_text(&mut self, text: &str) {
        self.drawn.push(text.to_string());
    }

    fn poll_event(&mut self) -> Option<GuiEvent> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_records_text() {
        let mut backend = W32Backend::new();
        backend.draw_text("hello");
        assert_eq!(backend.drawn, vec!["hello".to_string()]);
    }
}
