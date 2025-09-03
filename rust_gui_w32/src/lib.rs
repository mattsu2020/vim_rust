use rust_gui_core::backend::{GuiBackend, GuiEvent};
use std::collections::VecDeque;

#[cfg(windows)]
use windows::Win32::System::SystemInformation::GetTickCount;

/// Minimal Windows backend.  Real drawing is delegated to the platform APIs
/// but for now these methods record actions for testing purposes.
#[derive(Default)]
pub struct W32Backend {
    pub drawn: Vec<String>,
    pub events: VecDeque<GuiEvent>,
}

impl W32Backend {
    pub fn new() -> Self {
        Self { drawn: Vec::new(), events: VecDeque::new() }
    }

    /// Queue an event so it can later be retrieved by `poll_event`.
    pub fn push_event(&mut self, ev: GuiEvent) {
        self.events.push_back(ev);
    }
}

/// Wrap the Win32 `GetTickCount` API through the `windows` crate.
///
/// On non-Windows platforms this returns `0` so that the function can be
/// compiled and tested without depending on the actual Windows API.
pub fn get_tick_count() -> u32 {
    #[cfg(windows)]
    unsafe {
        GetTickCount()
    }
    #[cfg(not(windows))]
    {
        0
    }
}

impl GuiBackend for W32Backend {
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
    fn draw_and_queue_event() {
        let mut backend = W32Backend::new();
        backend.draw_text("hello");
        backend.push_event(GuiEvent::Click { x: 1, y: 2 });
        assert_eq!(backend.drawn, vec!["hello".to_string()]);
        assert_eq!(backend.poll_event(), Some(GuiEvent::Click { x: 1, y: 2 }));
    }

    #[test]
    fn tick_count() {
        // Ensure calling through the windows crate wrapper works on all targets.
        let _ = get_tick_count();
    }
}
