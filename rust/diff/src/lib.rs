use core::ffi::{c_char, c_int, c_long, c_void};

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

#[derive(Clone, Copy)]
enum DiffOp<'a> {
    Equal(&'a str),
    Insert(&'a str),
    Delete(&'a str),
}

fn myers_diff<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<DiffOp<'a>> {
    let n = a.len();
    let m = b.len();
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            if a[i] == b[j] {
                dp[i][j] = dp[i + 1][j + 1] + 1;
            } else {
                dp[i][j] = dp[i + 1][j].max(dp[i][j + 1]);
            }
        }
    }
    let mut i = 0;
    let mut j = 0;
    let mut ops = Vec::new();
    while i < n && j < m {
        if a[i] == b[j] {
            ops.push(DiffOp::Equal(a[i]));
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            ops.push(DiffOp::Delete(a[i]));
            i += 1;
        } else {
            ops.push(DiffOp::Insert(b[j]));
            j += 1;
        }
    }
    while i < n {
        ops.push(DiffOp::Delete(a[i]));
        i += 1;
    }
    while j < m {
        ops.push(DiffOp::Insert(b[j]));
        j += 1;
    }
    ops
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
        let ops = myers_diff(&a_lines, &b_lines);
        if let Some(cb) = (*ecb).out_line {
            for op in ops {
                let mut line = String::new();
                match op {
                    DiffOp::Equal(l) => { line.push(' '); line.push_str(l); }
                    DiffOp::Insert(l) => { line.push('+'); line.push_str(l); }
                    DiffOp::Delete(l) => { line.push('-'); line.push_str(l); }
                }
                let mut buf = mmbuffer_t { ptr: line.as_mut_ptr() as *mut c_char, size: line.len() as c_long };
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

    #[test]
    fn myers_algorithm() {
        let a = vec!["a\n", "b\n"];
        let b = vec!["a\n", "c\n"];
        let ops = myers_diff(&a, &b);
        assert!(ops.iter().any(|op| matches!(op, DiffOp::Delete("b\n"))));
        assert!(ops.iter().any(|op| matches!(op, DiffOp::Insert("c\n"))));
    }
}
