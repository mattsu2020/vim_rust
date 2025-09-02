use rust_gui_core::{GuiCore};
#[cfg(target_os = "linux")]
use rust_gui_core::backend::linux::LinuxBackend as Backend;
#[cfg(target_os = "windows")]
use rust_gui_core::backend::windows::WindowsBackend as Backend;
#[cfg(target_os = "macos")]
use rust_gui_core::backend::macos::MacBackend as Backend;
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
