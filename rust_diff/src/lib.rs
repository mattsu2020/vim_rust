use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long, c_void};
use std::process::Command;
use std::ptr;

use diff::{mmbuffer_t, mmfile_t, xdemitcb_t, xdl_diff};

#[repr(C)]
pub enum DiffMode {
    External = 0,
    Xdiff = 1,
}

unsafe extern "C" fn collect(priv_: *mut c_void, mb: *mut mmbuffer_t, nbuf: c_int) -> c_int {
    if priv_.is_null() || mb.is_null() {
        return -1;
    }
    let bufs = std::slice::from_raw_parts(mb, nbuf as usize);
    let out = &mut *(priv_ as *mut String);
    for b in bufs {
        let slice = std::slice::from_raw_parts((*b).ptr as *const u8, (*b).size as usize);
        out.push_str(std::str::from_utf8(slice).unwrap_or(""));
    }
    0
}

fn read_file(path: &str) -> Result<Vec<u8>, ()> {
    std::fs::read(path).map_err(|_| ())
}

fn diff_files_internal(file1: &str, file2: &str, mode: DiffMode) -> Result<CString, ()> {
    match mode {
        DiffMode::External => {
            let out = Command::new("diff")
                .arg("-u")
                .arg(file1)
                .arg(file2)
                .output()
                .map_err(|_| ())?;
            Ok(CString::new(String::from_utf8_lossy(&out.stdout).to_string()).unwrap())
        }
        DiffMode::Xdiff => {
            let a = read_file(file1)?;
            let b = read_file(file2)?;
            let mf1 = mmfile_t {
                ptr: a.as_ptr() as *const c_char,
                size: a.len() as c_long,
            };
            let mf2 = mmfile_t {
                ptr: b.as_ptr() as *const c_char,
                size: b.len() as c_long,
            };
            let mut output = String::new();
            let mut ecb = xdemitcb_t {
                priv_: &mut output as *mut _ as *mut c_void,
                out_hunk: None,
                out_line: Some(collect),
            };
            let res = unsafe { xdl_diff(&mf1, &mf2, ptr::null(), ptr::null(), &mut ecb) };
            if res != 0 {
                return Err(());
            }
            Ok(CString::new(output).unwrap())
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_diff_files(
    f1: *const c_char,
    f2: *const c_char,
    mode: DiffMode,
) -> *mut c_char {
    if f1.is_null() || f2.is_null() {
        return ptr::null_mut();
    }
    let file1 = unsafe { CStr::from_ptr(f1) }.to_string_lossy().into_owned();
    let file2 = unsafe { CStr::from_ptr(f2) }.to_string_lossy().into_owned();
    match diff_files_internal(&file1, &file2, mode) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rs_diff_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

#[no_mangle]
pub extern "C" fn rs_diff_update_screen() {
    // Placeholder for integrating with screen updating.  Currently this just logs.
    eprintln!("rs_diff_update_screen called");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::ffi::{CStr, CString};
    use std::fs::write;

    #[test]
    fn external_diff() {
        let dir = temp_dir();
        let f1 = dir.join("a.txt");
        let f2 = dir.join("b.txt");
        write(&f1, "a\nb\n").unwrap();
        write(&f2, "a\nc\n").unwrap();
        let c1 = CString::new(f1.to_str().unwrap()).unwrap();
        let c2 = CString::new(f2.to_str().unwrap()).unwrap();
        let ptr = rs_diff_files(c1.as_ptr(), c2.as_ptr(), DiffMode::External);
        assert!(!ptr.is_null());
        let diff = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
        rs_diff_free(ptr);
        assert!(diff.contains("-b"));
        assert!(diff.contains("+c"));
    }

    #[test]
    fn xdiff_diff() {
        let dir = temp_dir();
        let f1 = dir.join("c.txt");
        let f2 = dir.join("d.txt");
        write(&f1, "a\nb\n").unwrap();
        write(&f2, "a\nc\n").unwrap();
        let c1 = CString::new(f1.to_str().unwrap()).unwrap();
        let c2 = CString::new(f2.to_str().unwrap()).unwrap();
        let ptr = rs_diff_files(c1.as_ptr(), c2.as_ptr(), DiffMode::Xdiff);
        assert!(!ptr.is_null());
        let diff = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
        rs_diff_free(ptr);
        assert!(diff.contains("-b"));
        assert!(diff.contains("+c"));
    }
}
