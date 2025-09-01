use std::os::raw::c_int;

#[cfg(unix)]
mod imp {
    use super::*;
    use nix::libc;

    pub unsafe fn get_winsize(width: *mut c_int, height: *mut c_int) -> c_int {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == -1 {
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
}

#[cfg(windows)]
mod imp {
    use super::*;

    pub unsafe fn get_winsize(_width: *mut c_int, _height: *mut c_int) -> c_int {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_term_get_winsize(width: *mut c_int, height: *mut c_int) -> c_int {
    imp::get_winsize(width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_non_negative() {
        let mut w = 0;
        let mut h = 0;
        unsafe {
            let _ = rust_term_get_winsize(&mut w, &mut h);
        }
        assert!(w >= 0 && h >= 0);
    }
}
