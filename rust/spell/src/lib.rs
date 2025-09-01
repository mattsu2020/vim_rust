use std::os::raw::{c_int, c_uchar};

const WF_ONECAP: c_int = 0x02;
const WF_ALLCAP: c_int = 0x04;
const WF_KEEPCAP: c_int = 0x80;

#[no_mangle]
pub extern "C" fn captype(word: *const c_uchar, end: *const c_uchar) -> c_int {
    if word.is_null() {
        return 0;
    }
    unsafe {
        let len = if end.is_null() {
            let mut l = 0;
            while *word.add(l) != 0 { l += 1; }
            l
        } else {
            end.offset_from(word) as usize
        };
        let bytes = std::slice::from_raw_parts(word, len);
        // Find first alphabetic character
        let mut idx = 0;
        while idx < bytes.len() && !bytes[idx].is_ascii_alphabetic() {
            idx += 1;
        }
        if idx >= bytes.len() {
            return 0;
        }
        let first = bytes[idx];
        let mut firstcap = first.is_ascii_uppercase();
        let mut allcap = firstcap;
        idx += 1;
        let mut past_second = false;
        while idx < bytes.len() {
            let c = bytes[idx];
            idx += 1;
            if !c.is_ascii_alphabetic() {
                continue;
            }
            if !c.is_ascii_uppercase() {
                if past_second && allcap {
                    return WF_KEEPCAP;
                }
                allcap = false;
            } else if !allcap {
                return WF_KEEPCAP;
            }
            past_second = true;
        }
        if allcap {
            WF_ALLCAP
        } else if firstcap {
            WF_ONECAP
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn captype_basic() {
        let w = CString::new("vim").unwrap();
        assert_eq!(captype(w.as_ptr() as *const u8, std::ptr::null()), 0);
        let w = CString::new("Vim").unwrap();
        assert_eq!(captype(w.as_ptr() as *const u8, std::ptr::null()), WF_ONECAP);
        let w = CString::new("VIM").unwrap();
        assert_eq!(captype(w.as_ptr() as *const u8, std::ptr::null()), WF_ALLCAP);
        let w = CString::new("vIM").unwrap();
        assert_eq!(captype(w.as_ptr() as *const u8, std::ptr::null()), WF_KEEPCAP);
    }
}
