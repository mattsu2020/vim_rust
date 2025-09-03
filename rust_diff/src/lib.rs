use std::ffi::{CStr, CString, NulError};
use std::os::raw::{c_char, c_int, c_long, c_void};
use std::path::Path;
use std::process::Command;
use std::ptr;

use diff::{mmbuffer_t, mmfile_t, xdemitcb_t, xdl_diff};

#[derive(Debug)]
pub enum DiffError {
    Io(std::io::Error),
    Nul(NulError),
    XdiffFailed,
}

impl From<std::io::Error> for DiffError {
    fn from(e: std::io::Error) -> Self {
        DiffError::Io(e)
    }
}

impl From<NulError> for DiffError {
    fn from(e: NulError) -> Self {
        DiffError::Nul(e)
    }
}

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

fn read_file(path: &CStr) -> Result<Vec<u8>, DiffError> {
    let s = path.to_string_lossy();
    let p = Path::new(&*s);
    Ok(std::fs::read(p)?)
}

fn rs_diff_files_impl(f1: &CStr, f2: &CStr, mode: DiffMode) -> Result<*mut c_char, DiffError> {
    match mode {
        DiffMode::External => {
            let file1 = f1.to_string_lossy();
            let file2 = f2.to_string_lossy();
            let out = Command::new("diff")
                .arg("-u")
                .arg(&*file1)
                .arg(&*file2)
                .output()?;
            let cstr = CString::new(String::from_utf8_lossy(&out.stdout).to_string())?;
            Ok(cstr.into_raw())
        }
        DiffMode::Xdiff => {
            let a = read_file(f1)?;
            let b = read_file(f2)?;
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
                return Err(DiffError::XdiffFailed);
            }
            let cstr = CString::new(output)?;
            Ok(cstr.into_raw())
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
    let file1 = unsafe { CStr::from_ptr(f1) };
    let file2 = unsafe { CStr::from_ptr(f2) };
    match rs_diff_files_impl(file1, file2, mode) {
        Ok(ptr) => ptr,
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
