use rust_gui_core::{GuiCore, GuiEvent};
use rust_gui_gtk_x11::GtkX11Backend;

fn main() {
    let mut gui = GuiCore::new(GtkX11Backend::new());
    gui.draw_text("hello gtk_x11");
    gui.backend_mut().push_event(GuiEvent::Expose);
    gui.process_events(|ev| println!("event: {:?}", ev));
}
