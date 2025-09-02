#[cfg(target_os = "linux")]
use rust_gui_x11::X11Backend as Backend;
#[cfg(target_os = "macos")]
use rust_gui_core::backend::macos::MacBackend as Backend;
#[cfg(target_os = "windows")]
use rust_gui_core::backend::w32::W32Backend as Backend;
use rust_gui_core::GuiCore;
use rust_gui_core::GuiEvent;

/// Run the GUI.  This is exposed to the C code via `gui_rust.c`.
#[no_mangle]
pub extern "C" fn rs_gui_run() {
    let backend = Backend::new();
    let mut gui = GuiCore::new(backend);
    gui.draw_text("Vim Rust GUI");
    // Process any queued events; in this simple example we just ignore them.
    gui.process_events(|_e: GuiEvent| {});
}
