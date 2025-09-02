use std::os::raw::{c_char, c_int};

#[cfg(not(test))]
extern "C" {
    fn event_name2nr_rs(start: *const c_char, end: *mut *const c_char) -> c_int;
    fn event_key_rs(event: c_int) -> c_int;
    static mut p_ei: *const c_char;
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn event_name2nr_rs(start: *const c_char, end: *mut *const c_char) -> c_int {
    unsafe { *end = start; }
    -1
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn event_key_rs(_event: c_int) -> c_int { 0 }

#[cfg(test)]
#[no_mangle]
pub static mut p_ei: *const c_char = std::ptr::null();

unsafe fn to_lower(c: c_char) -> c_char {
    if c >= b'A' as c_char && c <= b'Z' as c_char {
        c - (b'A' as c_char) + (b'a' as c_char)
    } else {
        c
    }
}

unsafe fn is_all(p: *const c_char) -> bool {
    to_lower(*p) == b'a' as c_char
        && to_lower(*p.add(1)) == b'l' as c_char
        && to_lower(*p.add(2)) == b'l' as c_char
}

#[no_mangle]
pub unsafe extern "C" fn rs_event_ignored(event: c_int, ei: *const c_char) -> c_int {
    if ei.is_null() {
        return 0;
    }
    let mut p = ei;
    let mut ignored = false;
    while *p != 0 {
        let unignore = *p == b'-' as c_char;
        if unignore {
            p = p.add(1);
        }
        if is_all(p) && (*p.add(3) == 0 || *p.add(3) == b',' as c_char) {
            ignored = (p == p_ei) || event_key_rs(event) <= 0;
            if *p.add(3) == b',' as c_char {
                p = p.add(4);
            } else {
                p = p.add(3);
            }
        } else {
            let mut end: *const c_char = p;
            let ev = event_name2nr_rs(p, &mut end as *mut _);
            p = end;
            if ev == event {
                if unignore {
                    return 0;
                } else {
                    ignored = true;
                }
            }
        }
    }
    ignored as c_int
}

#[no_mangle]
pub unsafe extern "C" fn rs_check_ei(ei: *const c_char) -> c_int {
    if ei.is_null() {
        return 1;
    }
    let win = ei != p_ei;
    let mut p = ei;
    while *p != 0 {
        if is_all(p) && (*p.add(3) == 0 || *p.add(3) == b',' as c_char) {
            if *p.add(3) == b',' as c_char {
                p = p.add(4);
            } else {
                p = p.add(3);
            }
        } else {
            if *p == b'-' as c_char {
                p = p.add(1);
            }
            let mut end: *const c_char = p;
            let ev = event_name2nr_rs(p, &mut end as *mut _);
            if ev < 0 || (win && event_key_rs(ev) > 0) {
                return 1;
            }
            p = end;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn basic_all() {
        unsafe {
            let s = CString::new("all").unwrap();
            p_ei = s.as_ptr();
            assert_eq!(rs_event_ignored(0, p_ei), 1);
            assert_eq!(rs_check_ei(p_ei), 0);
            let s2 = CString::new("Foo").unwrap();
            assert_eq!(rs_check_ei(s2.as_ptr()), 1);
        }
    }
}
