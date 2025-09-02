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

#[cfg(target_os = "macos")]
pub mod macos;
