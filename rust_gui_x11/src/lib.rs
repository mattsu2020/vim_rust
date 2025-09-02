use rust_gui_core::backend::{GuiBackend, GuiEvent};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt as _, EventMask, KeyPressEvent, ButtonPressEvent, WindowClass, CreateWindowAux, CreateGCAux, Gcontext, Window};
use x11rb::rust_connection::RustConnection;

/// Backend implementation using the X11 protocol via x11rb.
pub struct X11Backend {
    conn: RustConnection,
    window: Window,
}

impl X11Backend {
    /// Establish a connection to the X server and create a simple window.
    pub fn new() -> Self {
        let (conn, screen_num) = x11rb::connect(None).expect("Failed to connect to X server");
        let screen = &conn.setup().roots[screen_num];
        let window = conn.generate_id().unwrap();
        conn.create_window(
            x11rb::COPY_FROM_PARENT as u8,
            window,
            screen.root,
            0,
            0,
            800,
            600,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().event_mask(
                EventMask::EXPOSURE | EventMask::KEY_PRESS | EventMask::BUTTON_PRESS,
            ),
        )
        .unwrap();
        conn.map_window(window).unwrap();
        conn.flush().unwrap();
        Self { conn, window }
    }
}

impl GuiBackend for X11Backend {
    fn draw_text(&mut self, text: &str) {
        let gc: Gcontext = self.conn.generate_id().unwrap();
        self.conn
            .create_gc(gc, self.window, &CreateGCAux::new())
            .unwrap();
        // Draw text at a fixed position.
        self.conn
            .image_text8(self.window, gc, 10, 20, text.as_bytes())
            .unwrap();
        self.conn.free_gc(gc).unwrap();
        self.conn.flush().unwrap();
    }

    fn poll_event(&mut self) -> Option<GuiEvent> {
        if let Ok(Some(event)) = self.conn.poll_for_event() {
            use x11rb::protocol::Event;
            match event {
                Event::KeyPress(KeyPressEvent { detail, .. }) => {
                    let ch = char::from_u32(detail.into()).unwrap_or('\0');
                    Some(GuiEvent::Key(ch))
                }
                Event::ButtonPress(ButtonPressEvent { event_x, event_y, .. }) => {
                    Some(GuiEvent::Click { x: event_x.into(), y: event_y.into() })
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
