use libc;
use std::os::raw::{c_char, c_int, c_long};

#[repr(C)]
pub struct mmfile_t {
    pub ptr: *const c_char,
    pub size: c_long,
}

const MATCH_CHAR_MAX_LEN: usize = 800;

fn matching_chars(s1: &[u8], s2: &[u8]) -> i32 {
    let s1len = std::cmp::min(s1.len(), MATCH_CHAR_MAX_LEN - 1);
    let s2len = std::cmp::min(s2.len(), MATCH_CHAR_MAX_LEN - 1);
    let mut matrix = [[0i32; MATCH_CHAR_MAX_LEN]; 2];
    let mut icur = 1usize;
    for i in 0..s1len {
        icur ^= 1;
        let (e1, e2) = if icur == 0 {
            let (a, b) = matrix.split_at_mut(1);
            (&mut a[0], &b[0])
        } else {
            let (a, b) = matrix.split_at_mut(1);
            (&mut b[0], &a[0])
        };
        for j in 0..s2len {
            if e2[j + 1] > e1[j + 1] {
                e1[j + 1] = e2[j + 1];
            }
            if e1[j] > e1[j + 1] {
                e1[j + 1] = e1[j];
            }
            if s1[i] == s2[j] && e2[j] + 1 > e1[j + 1] {
                e1[j + 1] = e2[j] + 1;
            }
        }
    }
    matrix[icur][s2len]
}

fn matching_chars_iwhite(s1: &[u8], s2: &[u8]) -> i32 {
    fn strip(input: &[u8]) -> Vec<u8> {
        input
            .iter()
            .copied()
            .filter(|c| *c != b' ' && *c != b'\t')
            .collect()
    }
    let p1 = strip(s1);
    let p2 = strip(s2);
    matching_chars(&p1, &p2)
}

unsafe fn lines_from_mmfile(m: &mmfile_t) -> Vec<&[u8]> {
    let slice = std::slice::from_raw_parts(m.ptr as *const u8, m.size as usize);
    let mut lines = Vec::new();
    let mut start = 0;
    for (i, &b) in slice.iter().enumerate() {
        if b == b'\n' {
            lines.push(&slice[start..i]);
            start = i + 1;
        }
    }
    if start < slice.len() {
        lines.push(&slice[start..]);
    }
    lines
}

#[no_mangle]
pub extern "C" fn linematch_nbuffers(
    diff_blk: *const *const mmfile_t,
    diff_len: *const c_int,
    ndiffs: usize,
    decisions: *mut *mut c_int,
    iwhite: c_int,
) -> usize {
    unsafe {
        if diff_blk.is_null() || diff_len.is_null() || decisions.is_null() {
            return 0;
        }
        let blocks = std::slice::from_raw_parts(diff_blk, ndiffs);
        let lens = std::slice::from_raw_parts(diff_len, ndiffs);
        if ndiffs != 2 {
            *decisions = std::ptr::null_mut();
            return 0;
        }
        let buf1 = &*blocks[0];
        let buf2 = &*blocks[1];
        let lines1 = lines_from_mmfile(buf1);
        let lines2 = lines_from_mmfile(buf2);
        let m = lens[0] as usize;
        let n = lens[1] as usize;
        let mut score = vec![0i32; (m + 1) * (n + 1)];
        let mut choice = vec![0i32; (m + 1) * (n + 1)];
        for i in 0..m {
            for j in 0..n {
                let idx = (i + 1) * (n + 1) + (j + 1);
                let mut best = score[i * (n + 1) + (j + 1)];
                let mut dec = 1; // skip buf1
                if score[(i + 1) * (n + 1) + j] > best {
                    best = score[(i + 1) * (n + 1) + j];
                    dec = 2; // skip buf2
                }
                let match_score = if iwhite != 0 {
                    matching_chars_iwhite(lines1[i], lines2[j])
                } else {
                    matching_chars(lines1[i], lines2[j])
                };
                if score[i * (n + 1) + j] + match_score > best {
                    best = score[i * (n + 1) + j] + match_score;
                    dec = 3; // compare
                }
                score[idx] = best;
                choice[idx] = dec;
            }
        }
        let mut i = m;
        let mut j = n;
        let mut path: Vec<c_int> = Vec::new();
        while i > 0 || j > 0 {
            let idx = i * (n + 1) + j;
            let dec = choice[idx] as c_int;
            if dec == 0 {
                break;
            }
            path.push(dec);
            if dec & 1 != 0 && i > 0 {
                i -= 1;
            }
            if dec & 2 != 0 && j > 0 {
                j -= 1;
            }
        }
        path.reverse();
        let len = path.len();
        if len == 0 {
            *decisions = std::ptr::null_mut();
            return 0;
        }
        let ptr = libc::malloc(len * std::mem::size_of::<c_int>()) as *mut c_int;
        if ptr.is_null() {
            *decisions = std::ptr::null_mut();
            return 0;
        }
        for (i, v) in path.iter().enumerate() {
            *ptr.add(i) = *v;
        }
        *decisions = ptr;
        len
    }
}
