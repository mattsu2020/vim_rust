use std::os::raw::c_int;
use nix::libc::{winsize, STDOUT_FILENO};

nix::ioctl_read_bad!(tiocgwinsz, nix::libc::TIOCGWINSZ, winsize);

pub unsafe fn get_winsize(width: *mut c_int, height: *mut c_int) -> c_int {
    let mut ws: winsize = std::mem::zeroed();
    if tiocgwinsz(STDOUT_FILENO, &mut ws).is_err() {
        return -1;
    }
    if !width.is_null() {
        *width = ws.ws_col as c_int;
    }
    if !height.is_null() {
        *height = ws.ws_row as c_int;
    }
    0
}
