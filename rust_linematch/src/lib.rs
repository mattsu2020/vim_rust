use core::ffi::{c_char, c_int, c_long};

#[repr(C)]
pub struct mmfile_t {
    pub ptr: *const c_char,
    pub size: c_long,
}

const LN_MAX_BUFS: usize = 8;
const LN_DECISION_MAX: usize = 255; // pow(2, LN_MAX_BUFS) - 1
const MATCH_CHAR_MAX_LEN: usize = 800;

fn line_len(m: &mmfile_t) -> usize {
    if m.ptr.is_null() || m.size <= 0 {
        return 0;
    }
    let slice = unsafe { std::slice::from_raw_parts(m.ptr as *const u8, m.size as usize) };
    match slice.iter().position(|&c| c == b'\n') {
        Some(pos) => pos,
        None => slice.len(),
    }
}

fn matching_chars(m1: &mmfile_t, m2: &mmfile_t) -> c_int {
    let s1len = line_len(m1).min(MATCH_CHAR_MAX_LEN - 1);
    let s2len = line_len(m2).min(MATCH_CHAR_MAX_LEN - 1);
    let s1 = unsafe { std::slice::from_raw_parts(m1.ptr as *const u8, s1len) };
    let s2 = unsafe { std::slice::from_raw_parts(m2.ptr as *const u8, s2len) };
    let mut matrix = [[0i32; MATCH_CHAR_MAX_LEN]; 2];
    let mut icur = 1usize;
    for i in 0..s1len {
        icur ^= 1;
        let (e1, e2) = if icur == 1 {
            (&mut matrix[1], &matrix[0])
        } else {
            (&mut matrix[0], &matrix[1])
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

fn matching_chars_iwhite(s1: &mmfile_t, s2: &mmfile_t) -> c_int {
    let mut p1 = [0u8; MATCH_CHAR_MAX_LEN];
    let mut p2 = [0u8; MATCH_CHAR_MAX_LEN];
    let mut m1p = mmfile_t {
        ptr: p1.as_ptr() as *const c_char,
        size: 0,
    };
    let mut m2p = mmfile_t {
        ptr: p2.as_ptr() as *const c_char,
        size: 0,
    };
    for (k, m) in [s1, s2].iter().enumerate() {
        let slice = unsafe {
            std::slice::from_raw_parts(
                (*m).ptr as *const u8,
                line_len(m).min(MATCH_CHAR_MAX_LEN - 1),
            )
        };
        let mut pi = 0;
        for &e in slice {
            if e != b' ' && e != b'\t' {
                if k == 0 {
                    p1[pi] = e;
                } else {
                    p2[pi] = e;
                }
                pi += 1;
            }
        }
        if k == 0 {
            m1p.size = pi as c_long;
        } else {
            m2p.size = pi as c_long;
        }
    }
    matching_chars(&m1p, &m2p)
}

fn count_n_matched_chars(sp: &[mmfile_t], iwhite: bool) -> c_int {
    let n = sp.len();
    let mut matched_chars = 0;
    let mut matched = 0;
    for i in 0..n {
        for j in (i + 1)..n {
            if sp[i].ptr.is_null() || sp[j].ptr.is_null() {
                continue;
            }
            matched += 1;
            matched_chars += if iwhite {
                matching_chars_iwhite(&sp[i], &sp[j])
            } else {
                matching_chars(&sp[i], &sp[j])
            };
        }
    }
    if matched >= 2 {
        matched_chars *= 2;
        matched_chars /= matched;
    }
    matched_chars
}

fn fastforward_buf_to_lnum(mut s: mmfile_t, lnum: c_int) -> mmfile_t {
    if s.ptr.is_null() || s.size <= 0 {
        return s;
    }
    for _ in 0..(lnum - 1) {
        if s.ptr.is_null() || s.size <= 0 {
            break;
        }
        let slice = unsafe { std::slice::from_raw_parts(s.ptr as *const u8, s.size as usize) };
        if let Some(idx) = slice.iter().position(|&b| b == b'\n') {
            s.ptr = unsafe { s.ptr.add(idx + 1) };
            s.size -= (idx + 1) as c_long;
        } else {
            s.ptr = core::ptr::null();
            s.size = 0;
            break;
        }
    }
    s
}

fn unwrap_indexes(values: &[c_int], diff_len: &[c_int], ndiffs: usize) -> usize {
    let mut num_unwrap_scalar = 1usize;
    for k in 0..ndiffs {
        num_unwrap_scalar *= (diff_len[k] + 1) as usize;
    }
    let mut path_idx = 0usize;
    for k in 0..ndiffs {
        num_unwrap_scalar /= (diff_len[k] + 1) as usize;
        let n = values[k] as usize;
        path_idx += num_unwrap_scalar * n;
    }
    path_idx
}

#[derive(Clone)]
struct DiffCmpPath {
    df_lev_score: c_int,
    df_path_n: usize,
    df_choice_mem: [c_int; LN_DECISION_MAX + 1],
    df_choice: [c_int; LN_DECISION_MAX],
    df_decision: [usize; LN_DECISION_MAX],
    df_optimal_choice: usize,
}

impl DiffCmpPath {
    fn new() -> Self {
        Self {
            df_lev_score: 0,
            df_path_n: 0,
            df_choice_mem: [-1; LN_DECISION_MAX + 1],
            df_choice: [0; LN_DECISION_MAX],
            df_decision: [0; LN_DECISION_MAX],
            df_optimal_choice: 0,
        }
    }
}

fn try_possible_paths(
    df_iters: &[c_int; LN_MAX_BUFS],
    paths: &[usize],
    path_idx: usize,
    choice: &mut c_int,
    diffcmppath: &mut [DiffCmpPath],
    diff_len: &[c_int],
    ndiffs: usize,
    diff_blk: &[&mmfile_t],
    iwhite: bool,
) {
    if path_idx == paths.len() {
        if *choice > 0 {
            let mut from_vals = [0i32; LN_MAX_BUFS];
            let to_vals = df_iters;
            let mut mm = [mmfile_t {
                ptr: core::ptr::null(),
                size: 0,
            }; LN_MAX_BUFS];
            let mut current_lines = [mmfile_t {
                ptr: core::ptr::null(),
                size: 0,
            }; LN_MAX_BUFS];
            for k in 0..ndiffs {
                from_vals[k] = df_iters[k];
                if (*choice & (1 << k)) != 0 {
                    from_vals[k] -= 1;
                    mm[k] = fastforward_buf_to_lnum(*diff_blk[k], df_iters[k]);
                }
                current_lines[k] = mm[k];
            }
            let unwrapped_idx_from = unwrap_indexes(&from_vals[..ndiffs], diff_len, ndiffs);
            let unwrapped_idx_to = unwrap_indexes(&to_vals[..ndiffs], diff_len, ndiffs);
            let matched_chars = count_n_matched_chars(&current_lines[..ndiffs], iwhite);
            let score = diffcmppath[unwrapped_idx_from].df_lev_score + matched_chars;
            if score > diffcmppath[unwrapped_idx_to].df_lev_score {
                diffcmppath[unwrapped_idx_to].df_path_n = 1;
                diffcmppath[unwrapped_idx_to].df_decision[0] = unwrapped_idx_from;
                diffcmppath[unwrapped_idx_to].df_choice[0] = *choice;
                diffcmppath[unwrapped_idx_to].df_lev_score = score;
            } else if score == diffcmppath[unwrapped_idx_to].df_lev_score {
                let k = diffcmppath[unwrapped_idx_to].df_path_n;
                diffcmppath[unwrapped_idx_to].df_path_n += 1;
                diffcmppath[unwrapped_idx_to].df_decision[k] = unwrapped_idx_from;
                diffcmppath[unwrapped_idx_to].df_choice[k] = *choice;
            }
        }
        return;
    }
    let bit_place = paths[path_idx];
    *choice |= 1 << bit_place;
    try_possible_paths(
        df_iters,
        paths,
        path_idx + 1,
        choice,
        diffcmppath,
        diff_len,
        ndiffs,
        diff_blk,
        iwhite,
    );
    *choice &= !(1 << bit_place);
    try_possible_paths(
        df_iters,
        paths,
        path_idx + 1,
        choice,
        diffcmppath,
        diff_len,
        ndiffs,
        diff_blk,
        iwhite,
    );
}

fn populate_tensor(
    df_iters: &mut [c_int; LN_MAX_BUFS],
    ch_dim: usize,
    diffcmppath: &mut [DiffCmpPath],
    diff_len: &[c_int],
    ndiffs: usize,
    diff_blk: &[&mmfile_t],
    iwhite: bool,
) {
    if ch_dim == ndiffs {
        let mut npaths = 0usize;
        let mut paths = [0usize; LN_MAX_BUFS];
        for j in 0..ndiffs {
            if df_iters[j] > 0 {
                paths[npaths] = j;
                npaths += 1;
            }
        }
        let mut choice = 0;
        let unwrapper_idx_to = unwrap_indexes(&df_iters[..ndiffs], diff_len, ndiffs);
        diffcmppath[unwrapper_idx_to].df_lev_score = -1;
        try_possible_paths(
            df_iters,
            &paths[..npaths],
            0,
            &mut choice,
            diffcmppath,
            diff_len,
            ndiffs,
            diff_blk,
            iwhite,
        );
        return;
    }
    for i in 0..=diff_len[ch_dim] {
        df_iters[ch_dim] = i;
        populate_tensor(
            df_iters,
            ch_dim + 1,
            diffcmppath,
            diff_len,
            ndiffs,
            diff_blk,
            iwhite,
        );
    }
}

fn test_charmatch_paths(diffcmppath: &mut [DiffCmpPath], idx: usize, lastdecision: c_int) -> usize {
    if diffcmppath[idx].df_choice_mem[lastdecision as usize] == -1 {
        if diffcmppath[idx].df_path_n == 0 {
            diffcmppath[idx].df_choice_mem[lastdecision as usize] = 0;
        } else {
            let mut minimum_turns = usize::MAX;
            for i in 0..diffcmppath[idx].df_path_n {
                let next_idx = diffcmppath[idx].df_decision[i];
                let t = test_charmatch_paths(diffcmppath, next_idx, diffcmppath[idx].df_choice[i])
                    + if lastdecision != diffcmppath[idx].df_choice[i] {
                        1
                    } else {
                        0
                    };
                if t < minimum_turns {
                    diffcmppath[idx].df_optimal_choice = i;
                    minimum_turns = t;
                }
            }
            diffcmppath[idx].df_choice_mem[lastdecision as usize] = minimum_turns as c_int;
        }
    }
    diffcmppath[idx].df_choice_mem[lastdecision as usize] as usize
}

pub fn linematch_nbuffers(diff_blk: &[&mmfile_t], diff_len: &[c_int], iwhite: bool) -> Vec<c_int> {
    let ndiffs = diff_blk.len();
    assert!(ndiffs <= LN_MAX_BUFS);
    let mut memsize = 1usize;
    let mut memsize_decisions = 0usize;
    for i in 0..ndiffs {
        assert!(diff_len[i] >= 0);
        memsize *= (diff_len[i] + 1) as usize;
        memsize_decisions += diff_len[i] as usize;
    }
    let mut diffcmppath = vec![DiffCmpPath::new(); memsize];
    let mut df_iters = [0i32; LN_MAX_BUFS];
    populate_tensor(
        &mut df_iters,
        0,
        &mut diffcmppath,
        diff_len,
        ndiffs,
        diff_blk,
        iwhite,
    );
    let u = unwrap_indexes(diff_len, diff_len, ndiffs);
    test_charmatch_paths(&mut diffcmppath, u, 0);
    let mut node_idx = u;
    let mut decisions = Vec::with_capacity(memsize_decisions);
    while diffcmppath[node_idx].df_path_n > 0 {
        let j = diffcmppath[node_idx].df_optimal_choice;
        decisions.push(diffcmppath[node_idx].df_choice[j]);
        node_idx = diffcmppath[node_idx].df_decision[j];
    }
    decisions.reverse();
    decisions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_linematch() {
        let a = b"a\nb\n";
        let b = b"a\nc\n";
        let m1 = mmfile_t {
            ptr: a.as_ptr() as *const c_char,
            size: a.len() as c_long,
        };
        let m2 = mmfile_t {
            ptr: b.as_ptr() as *const c_char,
            size: b.len() as c_long,
        };
        let diff_blk = [&m1, &m2];
        let diff_len = [2, 2];
        let res = linematch_nbuffers(&diff_blk, &diff_len, false);
        assert_eq!(res.len(), 2);
    }
}
