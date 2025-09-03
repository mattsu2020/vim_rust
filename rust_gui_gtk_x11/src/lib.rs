use rust_gui_core::backend::{GuiBackend, GuiEvent};
use rust_gui_gtk::GtkBackend;
#[cfg(feature = "use_x11")]
use rust_gui_x11::X11Backend;

/// Backend combining GTK event handling with X11 drawing.
///
/// This minimalist implementation delegates drawing to the X11 backend
/// while reusing the event queue from the GTK backend.  It serves as a
/// placeholder for the original `gui_gtk_x11.c` functionality.
pub struct GtkX11Backend {
    gtk: GtkBackend,
    #[cfg(feature = "use_x11")]
    x11: X11Backend,
}

impl GtkX11Backend {
    pub fn new() -> Self {
        Self {
            gtk: GtkBackend::new(),
            #[cfg(feature = "use_x11")]
            x11: X11Backend::new(),
        }
    }

    /// Queue an event as handled by the GTK layer.
    pub fn push_event(&mut self, ev: GuiEvent) {
        self.gtk.push_event(ev);
    }
}

impl GuiBackend for GtkX11Backend {
    fn draw_text(&mut self, text: &str) {
        #[cfg(feature = "use_x11")]
        {
            self.x11.draw_text(text);
        }
        #[cfg(not(feature = "use_x11"))]
        {
            self.gtk.draw_text(text);
        }
    }

    fn poll_event(&mut self) -> Option<GuiEvent> {
        self.gtk.poll_event()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine_backends() {
        let mut backend = GtkX11Backend::new();
        backend.draw_text("mixed");
        backend.push_event(GuiEvent::Expose);
        assert_eq!(backend.poll_event(), Some(GuiEvent::Expose));
    }
}
