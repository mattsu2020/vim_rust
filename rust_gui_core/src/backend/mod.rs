/// Basic events that can be produced by a GUI backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiEvent {
    /// A key was pressed.
    Key(char),
    /// Mouse click at the given coordinates.
    Click { x: i32, y: i32 },
}

/// Abstraction over platform specific drawing and event handling.
pub trait GuiBackend {
    /// Draw the provided text.  The exact location and font are
    /// backend specific and outside the scope of this example.
    fn draw_text(&mut self, text: &str);

    /// Return the next pending event if there is one.
    fn poll_event(&mut self) -> Option<GuiEvent>;
}

#[cfg(target_os = "linux")]
pub mod linux {
    use super::{GuiBackend, GuiEvent};
    use std::collections::VecDeque;

    /// Simple backend used for tests and non-GUI environments on Linux.
    /// Drawing operations are recorded and events are stored in a queue.
    #[derive(Default)]
    pub struct LinuxBackend {
        pub drawn: Vec<String>,
        pub events: VecDeque<GuiEvent>,
    }

    impl LinuxBackend {
        pub fn new() -> Self {
            Self { drawn: Vec::new(), events: VecDeque::new() }
        }

        /// Queue an event for later processing; primarily used in tests.
        pub fn push_event(&mut self, ev: GuiEvent) {
            self.events.push_back(ev);
        }
    }

    impl GuiBackend for LinuxBackend {
        fn draw_text(&mut self, text: &str) {
            self.drawn.push(text.to_string());
        }

        fn poll_event(&mut self) -> Option<GuiEvent> {
            self.events.pop_front()
        }
    }
}

#[cfg(target_os = "windows")]
pub mod windows {
    use super::{GuiBackend, GuiEvent};

    #[derive(Default)]
    pub struct WindowsBackend;

    impl WindowsBackend {
        pub fn new() -> Self { Self }
    }

    impl GuiBackend for WindowsBackend {
        fn draw_text(&mut self, _text: &str) {}
        fn poll_event(&mut self) -> Option<GuiEvent> { None }
    }
}

#[cfg(target_os = "macos")]
pub mod macos {
    use super::{GuiBackend, GuiEvent};

    #[derive(Default)]
    pub struct MacBackend;

    impl MacBackend {
        pub fn new() -> Self { Self }
    }

    impl GuiBackend for MacBackend {
        fn draw_text(&mut self, _text: &str) {}
        fn poll_event(&mut self) -> Option<GuiEvent> { None }
    }
}
