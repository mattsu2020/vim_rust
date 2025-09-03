use rust_gui_core::{GuiCore, GuiEvent};
use rust_gui_motif::MotifBackend;

fn main() {
    // Set up the core with the Motif backend
    let mut gui = GuiCore::new(MotifBackend::new());
    gui.draw_text("hello motif");
    gui.backend_mut().push_event(GuiEvent::Key('q'));
    gui.process_events(|ev| println!("event: {:?}", ev));
}
