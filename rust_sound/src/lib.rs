use std::os::raw::{c_char, c_int, c_long};

#[no_mangle]
pub extern "C" fn rs_sound_playevent(name: *const c_char) -> c_int {
    if name.is_null() {
        return -1;
    }
    #[cfg(unix)]
    {
        rust_os_unix::os_sound_playevent(name)
    }
    #[cfg(windows)]
    {
        rust_os_win32::os_sound_playevent(name)
    }
    #[cfg(all(not(unix), not(windows)))]
    {
        let _ = rust_os_api::play_beep();
        0
    }
}

#[no_mangle]
pub extern "C" fn rs_sound_playfile(path: *const c_char) -> c_int {
    if path.is_null() {
        return -1;
    }
    #[cfg(unix)]
    {
        rust_os_unix::os_sound_playfile(path)
    }
    #[cfg(windows)]
    {
        rust_os_win32::os_sound_playfile(path)
    }
    #[cfg(all(not(unix), not(windows)))]
    {
        -1
    }
}

#[no_mangle]
pub extern "C" fn rs_sound_stop(id: c_long) {
    #[cfg(unix)]
    {
        rust_os_unix::os_sound_stop(id);
    }
    #[cfg(windows)]
    {
        rust_os_win32::os_sound_stop(id);
    }
}

#[no_mangle]
pub extern "C" fn rs_sound_clear() {
    #[cfg(unix)]
    {
        rust_os_unix::os_sound_clear();
    }
    #[cfg(windows)]
    {
        rust_os_win32::os_sound_clear();
    }
}

#[no_mangle]
pub extern "C" fn has_any_sound_callback() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn has_sound_callback_in_queue() -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn invoke_sound_callback() {}

#[no_mangle]
pub extern "C" fn call_sound_callback(_cb: *mut std::ffi::c_void, _id: c_long, _result: c_int) {}

#[no_mangle]
pub extern "C" fn delete_sound_callback(_cb: *mut std::ffi::c_void) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn play_event_beep() {
        let c = CString::new("beep").unwrap();
        // Should succeed even if no real sound is played.
        assert_eq!(rs_sound_playevent(c.as_ptr()), 0);
    }
}
