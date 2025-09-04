use libc::{c_int};
use rust_ui::{flush, init, ScreenBuffer};

#[no_mangle]
pub extern "C" fn rs_drawscreen_init(_screen: *mut ScreenBuffer, width: c_int, height: c_int) {
    init(width as usize, height as usize);
}

#[no_mangle]
pub extern "C" fn rs_update_screen(_typ: c_int) {
    flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_update() {
        rs_drawscreen_init(std::ptr::null_mut(), 80, 24);
        rs_update_screen(1);
    }
}
