use libc::c_int;
use rust_gui_core::GuiCore;
use std::sync::{Mutex, OnceLock};

// Store screen dimensions provided by C side and a handle to the GUI backend.
static SCREEN_SIZE: OnceLock<(c_int, c_int)> = OnceLock::new();
static GUI: OnceLock<Mutex<GuiCore<Backend>>> = OnceLock::new();

#[cfg(feature = "x11")]
use rust_gui_x11::X11Backend as Backend;
#[cfg(feature = "w32")]
use rust_gui_w32::W32Backend as Backend;

#[no_mangle]
pub extern "C" fn rs_drawscreen_init(width: c_int, height: c_int) {
    let _ = SCREEN_SIZE.set((width, height));
    let backend = Backend::new();
    let gui = GuiCore::new(backend);
    let _ = GUI.set(Mutex::new(gui));
}

#[no_mangle]
pub extern "C" fn rs_update_screen(typ: c_int) {
    if let Some((w, h)) = SCREEN_SIZE.get() {
        eprintln!("update_screen type={} size={}x{}", typ, w, h);
    } else {
        eprintln!("update_screen called before init: type={}", typ);
    }

    if let Some(gui) = GUI.get() {
        let mut gui = gui.lock().unwrap();
        gui.draw_text(&format!("update_screen type={typ}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_update() {
        rs_drawscreen_init(80, 24);
        rs_update_screen(1);
        assert!(SCREEN_SIZE.get().is_some());

        // When using the w32 backend the drawn text is recorded and can be
        // inspected here.  This verifies that `rs_update_screen` forwards to
        // the backend through the common GUI interface.
        #[cfg(feature = "w32")]
        {
            let drawn = GUI
                .get()
                .unwrap()
                .lock()
                .unwrap()
                .backend_mut()
                .drawn
                .clone();
            assert_eq!(drawn, vec!["update_screen type=1".to_string()]);
        }
    }
}
