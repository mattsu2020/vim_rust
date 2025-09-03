use libc::{c_char, c_int};
use rust_gui_core::GuiCore;
use rust_screen::{self, ScreenBuffer};
use std::ffi::CStr;
use std::sync::{Mutex, OnceLock};

// Store screen dimensions provided by C side and a handle to the GUI backend.
static SCREEN_SIZE: OnceLock<(c_int, c_int)> = OnceLock::new();
static GUI: OnceLock<Mutex<GuiCore<Backend>>> = OnceLock::new();
static SCREEN: OnceLock<usize> = OnceLock::new();

#[cfg(feature = "x11")]
use rust_gui_x11::X11Backend as Backend;
#[cfg(feature = "w32")]
use rust_gui_w32::W32Backend as Backend;

#[no_mangle]
pub extern "C" fn rs_drawscreen_init(screen: *mut ScreenBuffer, width: c_int, height: c_int) {
    let _ = SCREEN.set(screen as usize);
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

    if let Some(&screen) = SCREEN.get() {
        rust_screen::rs_screen_flush(screen as *mut ScreenBuffer, Some(flush_callback));
    }
}

extern "C" fn flush_callback(row: c_int, text: *const c_char, _attr: *const u8, _len: c_int) {
    if let Some(gui) = GUI.get() {
        let s = unsafe { CStr::from_ptr(text).to_string_lossy().into_owned() };
        let mut gui = gui.lock().unwrap();
        gui.draw_text(&format!("row {row}: {s}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn init_and_update() {
        let sb = rust_screen::rs_screen_new(80, 24);
        rs_drawscreen_init(sb, 80, 24);
        let text = CString::new("hello").unwrap();
        rust_screen::rs_screen_draw_text(sb, 0, 0, text.as_ptr(), 1);
        rs_update_screen(1);
        assert!(SCREEN_SIZE.get().is_some());

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
            assert_eq!(drawn.len(), 1);
            assert_eq!(drawn[0].trim_end(), "row 0: hello");
        }

        rust_screen::rs_screen_free(sb);
    }
}
