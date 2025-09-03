use core::ffi::{c_char, c_int, c_long, c_void, c_uchar};
use similar::{ChangeTag, TextDiff};

#[repr(C)]
pub struct mmfile_t {
    pub ptr: *const c_char,
    pub size: c_long,
}

#[repr(C)]
pub struct mmbuffer_t {
    pub ptr: *mut c_char,
    pub size: c_long,
}

#[repr(C)]
pub struct xpparam_t {
    pub flags: u64,
    pub anchors: *mut *mut c_char,
    pub anchors_nr: usize,
}

#[repr(C)]
pub struct xdemitcb_t {
    pub priv_: *mut c_void,
    pub out_hunk: Option<unsafe extern "C" fn(*mut c_void, c_long, c_long, c_long, c_long, *const c_char, c_long) -> c_int>,
    pub out_line: Option<unsafe extern "C" fn(*mut c_void, *mut mmbuffer_t, c_int) -> c_int>,
}

#[repr(C)]
pub struct xdemitconf_t {
    pub ctxlen: c_long,
    pub interhunkctxlen: c_long,
    pub flags: u64,
    pub find_func: Option<unsafe extern "C" fn(*const c_char, c_long, *mut c_char, c_long, *mut c_void) -> c_long>,
    pub find_func_priv: *mut c_void,
    pub hunk_func: Option<unsafe extern "C" fn(c_long, c_long, c_long, c_long, *mut c_void) -> c_int>,
}

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

/// Perform a diff between two mmfiles and emit the result through callbacks.
/// This mirrors a small subset of the libxdiff `xdl_diff` interface.
///
/// Safety: all pointers must be valid for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn xdl_diff(
    mf1: *const mmfile_t,
    mf2: *const mmfile_t,
    _xpp: *const xpparam_t,
    _xecfg: *const xdemitconf_t,
    ecb: *mut xdemitcb_t,
) -> c_int {
    let res = std::panic::catch_unwind(|| {
        if mf1.is_null() || mf2.is_null() || ecb.is_null() {
            return -1;
        }
        let a_slice = std::slice::from_raw_parts((*mf1).ptr as *const u8, (*mf1).size as usize);
        let b_slice = std::slice::from_raw_parts((*mf2).ptr as *const u8, (*mf2).size as usize);
        let a_str = std::str::from_utf8(a_slice).unwrap_or("");
        let b_str = std::str::from_utf8(b_slice).unwrap_or("");
        let a_lines: Vec<&str> = if a_str.is_empty() { Vec::new() } else { a_str.split_inclusive('\n').collect() };
        let b_lines: Vec<&str> = if b_str.is_empty() { Vec::new() } else { b_str.split_inclusive('\n').collect() };
        let diff = TextDiff::configure().diff_slices(&a_lines, &b_lines);
        if let Some(cb) = (*ecb).out_line {
            for change in diff.iter_all_changes() {
                let val = change.value();
                let mut line = Vec::with_capacity(val.len() + 1);
                line.push(match change.tag() {
                    ChangeTag::Equal => b' ',
                    ChangeTag::Insert => b'+',
                    ChangeTag::Delete => b'-',
                });
                line.extend_from_slice(val.as_bytes());
                let mut buf = mmbuffer_t {
                    ptr: line.as_mut_ptr() as *mut c_char,
                    size: line.len() as c_long,
                };
                cb((*ecb).priv_, &mut buf, 1);
            }
        }
        0
    });
    match res {
        Ok(v) => v,
        Err(_) => -1,
    }
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
        let mf1 = mmfile_t { ptr: a.as_ptr() as *const c_char, size: a.len() as c_long };
        let mf2 = mmfile_t { ptr: b.as_ptr() as *const c_char, size: b.len() as c_long };
        let mut output = String::new();
        let mut ecb = xdemitcb_t { priv_: &mut output as *mut _ as *mut c_void, out_hunk: None, out_line: Some(collect) };
        let res = unsafe { xdl_diff(&mf1, &mf2, std::ptr::null(), std::ptr::null(), &mut ecb) };
        assert_eq!(res, 0);
        assert!(output.contains("-b"));
        assert!(output.contains("+c"));
    }
}
