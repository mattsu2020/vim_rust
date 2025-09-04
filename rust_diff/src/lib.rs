use core::ffi::{c_char, c_int, c_long, c_uchar, c_void};
use rust_linematch as lm;
use rust_linematch::mmfile_t as lm_mmfile_t;
use rust_xdiff::{
    mmbuffer_t, mmfile_t, xdemitcb_t, xdemitconf_t, xdl_diff as rs_xdl_diff, xpparam_t,
};

#[repr(C)]
pub struct garray_T {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

#[repr(C)]
pub struct diffout_T {
    pub dout_fname: *mut c_char,
    pub dout_ga: garray_T,
}

#[repr(C)]
pub struct diffhunk_T {
    pub lnum_orig: c_long,
    pub count_orig: c_long,
    pub lnum_new: c_long,
    pub count_new: c_long,
}

extern "C" {
    fn ga_concat_len(gap: *mut garray_T, s: *const c_uchar, len: usize);
    fn ga_grow(gap: *mut garray_T, n: c_int) -> c_int;
    fn alloc(size: usize) -> *mut c_void;
    fn vim_free(ptr: *mut c_void);
}

#[no_mangle]
pub unsafe extern "C" fn xdl_diff(
    mf1: *const mmfile_t,
    mf2: *const mmfile_t,
    xpp: *const xpparam_t,
    xecfg: *const xdemitconf_t,
    ecb: *mut xdemitcb_t,
) -> c_int {
    rs_xdl_diff(mf1, mf2, xpp, xecfg, ecb)
}

#[no_mangle]
pub unsafe extern "C" fn xdiff_out_unified(
    priv_: *mut c_void,
    mb: *mut mmbuffer_t,
    nbuf: c_int,
) -> c_int {
    if priv_.is_null() || mb.is_null() {
        return -1;
    }
    let dout = &mut *(priv_ as *mut diffout_T);
    let bufs = std::slice::from_raw_parts(mb, nbuf as usize);
    for b in bufs {
        ga_concat_len(&mut dout.dout_ga, b.ptr as *const c_uchar, b.size as usize);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn xdiff_out_indices(
    start_a: c_long,
    count_a: c_long,
    start_b: c_long,
    count_b: c_long,
    priv_: *mut c_void,
) -> c_int {
    if priv_.is_null() {
        return -1;
    }
    let dout = &mut *(priv_ as *mut diffout_T);
    let p = alloc(std::mem::size_of::<diffhunk_T>()) as *mut diffhunk_T;
    if p.is_null() {
        return -1;
    }
    if ga_grow(&mut dout.dout_ga, 1) == 0 {
        vim_free(p as *mut c_void);
        return -1;
    }
    (*p).lnum_orig = start_a + 1;
    (*p).count_orig = count_a;
    (*p).lnum_new = start_b + 1;
    (*p).count_new = count_b;
    let data = dout.dout_ga.ga_data as *mut *mut diffhunk_T;
    *data.add(dout.dout_ga.ga_len as usize) = p;
    dout.dout_ga.ga_len += 1;
    0
}

#[no_mangle]
pub unsafe extern "C" fn linematch_nbuffers(
    diff_blk: *const *const mmfile_t,
    diff_len: *const c_int,
    ndiffs: usize,
    decisions: *mut *mut c_int,
    iwhite: c_int,
) -> usize {
    if diff_blk.is_null() || diff_len.is_null() || decisions.is_null() {
        return 0;
    }
    let blk = std::slice::from_raw_parts(diff_blk, ndiffs);
    let lens = std::slice::from_raw_parts(diff_len, ndiffs);
    let mut refs: Vec<&lm_mmfile_t> = Vec::with_capacity(ndiffs);
    for &p in blk {
        refs.push(&*(p as *const lm_mmfile_t));
    }
    let vec = lm::linematch_nbuffers(&refs, lens, iwhite != 0);
    let size = vec.len() * std::mem::size_of::<c_int>();
    let ptr = alloc(size) as *mut c_int;
    if ptr.is_null() {
        *decisions = core::ptr::null_mut();
        return 0;
    }
    std::ptr::copy_nonoverlapping(vec.as_ptr(), ptr, vec.len());
    *decisions = ptr;
    vec.len()
}

#[no_mangle]
pub extern "C" fn rs_diff_update_screen() {
    eprintln!("rs_diff_update_screen called");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::c_void;

    unsafe extern "C" fn collect(priv_: *mut c_void, buf: *mut mmbuffer_t, _nr: c_int) -> c_int {
        let out = &mut *(priv_ as *mut String);
        let slice = std::slice::from_raw_parts((*buf).ptr as *const u8, (*buf).size as usize);
        out.push_str(std::str::from_utf8(slice).unwrap());
        0
    }

    #[test]
    fn diff_simple() {
        let a = b"a\nb\n";
        let b = b"a\nc\n";
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
        let res = unsafe { xdl_diff(&mf1, &mf2, std::ptr::null(), std::ptr::null(), &mut ecb) };
        assert_eq!(res, 0);
        assert!(output.contains("-b"));
        assert!(output.contains("+c"));
    }
}
