use libc::c_int;
use std::sync::OnceLock;

// Store screen dimensions provided by C side.
static SCREEN_SIZE: OnceLock<(c_int, c_int)> = OnceLock::new();

#[no_mangle]
pub extern "C" fn rs_drawscreen_init(width: c_int, height: c_int) {
    let _ = SCREEN_SIZE.set((width, height));
}

#[no_mangle]
pub extern "C" fn rs_update_screen(typ: c_int) {
    if let Some((w, h)) = SCREEN_SIZE.get() {
        eprintln!("update_screen type={} size={}x{}", typ, w, h);
    } else {
        eprintln!("update_screen called before init: type={}", typ);
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
    }
}
