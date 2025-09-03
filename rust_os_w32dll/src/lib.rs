#![cfg(windows)]

use std::os::raw::c_void;

extern "C" {
    fn SaveInst(h_inst: usize);
}

/// Minimal `DllMain` that stores the instance handle on process attach.
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinst_dll: usize,
    fdw_reason: u32,
    _lpv_reserved: *mut c_void,
) -> i32 {
    const DLL_PROCESS_ATTACH: u32 = 1;
    if fdw_reason == DLL_PROCESS_ATTACH {
        SaveInst(hinst_dll);
    }
    1 // TRUE
}

#[cfg(test)]
mod tests {
    use super::*;
    #[no_mangle]
    extern "C" fn SaveInst(_h: usize) {}


    #[test]
    fn calls_dllmain() {
        unsafe {
            assert_eq!(DllMain(0, 1, std::ptr::null_mut()), 1);
        }
    }
}
