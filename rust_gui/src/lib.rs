#[cfg(feature = "winit")]
use rust_gui_winit::WinitBackend as Backend;
#[cfg(all(target_os = "macos", not(feature = "winit")))]
use rust_gui_core::backend::macos::MacBackend as Backend;
#[cfg(all(target_os = "windows", not(feature = "winit")))]
use rust_gui_w32::W32Backend as Backend;
#[cfg(all(target_os = "linux", not(feature = "winit"), feature = "gtk"))]
use rust_gui_gtk::GtkBackend as Backend;
#[cfg(all(target_os = "linux", not(feature = "winit"), not(feature = "gtk")))]
use rust_gui_x11::X11Backend as Backend;
use rust_gui_core::GuiBackend;
use rust_gui_core::GuiCore;
use rust_gui_core::GuiEvent;

/// Run the GUI.  This is exposed to the C code via `gui_rust.c`.
#[no_mangle]
pub extern "C" fn rs_gui_run() {
    let backend = Backend::new();
    let mut gui = GuiCore::new(backend);
    gui.draw_text("Vim Rust GUI");
    // Redraw the window whenever an expose event is received.
    while let Some(ev) = gui.backend_mut().poll_event() {
        if let GuiEvent::Expose = ev {
            gui.draw_text("Vim Rust GUI");
        }
    }
}
