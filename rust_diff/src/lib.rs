use core::ffi::{c_char, c_int, c_long, c_void};
use rust_xdiff::{mmfile_t, xdemitcb_t, xdemitconf_t, xdl_diff as rs_xdl_diff, xpparam_t};

// Diff option flags, kept in sync with the historical C implementation.
pub const DIFF_FILLER: u64 = 0x001;
pub const DIFF_IBLANK: u64 = 0x002;
pub const DIFF_ICASE: u64 = 0x004;
pub const DIFF_IWHITE: u64 = 0x008;
pub const DIFF_IWHITEALL: u64 = 0x010;
pub const DIFF_IWHITEEOL: u64 = 0x020;
pub const DIFF_HORIZONTAL: u64 = 0x040;
pub const DIFF_VERTICAL: u64 = 0x080;
pub const DIFF_HIDDEN_OFF: u64 = 0x100;
pub const DIFF_INTERNAL: u64 = 0x200;
pub const DIFF_CLOSE_OFF: u64 = 0x400;
pub const DIFF_FOLLOWWRAP: u64 = 0x800;
pub const DIFF_LINEMATCH: u64 = 0x1000;
pub const DIFF_INLINE_NONE: u64 = 0x2000;
pub const DIFF_INLINE_SIMPLE: u64 = 0x4000;
pub const DIFF_INLINE_CHAR: u64 = 0x8000;
pub const DIFF_INLINE_WORD: u64 = 0x10000;
pub const DIFF_ANCHOR: u64 = 0x20000;

const ALL_INLINE: u64 = DIFF_INLINE_NONE | DIFF_INLINE_SIMPLE | DIFF_INLINE_CHAR | DIFF_INLINE_WORD;

// Flags for the underlying xdiff engine.
pub const XDF_NEED_MINIMAL: u64 = 1 << 0;
pub const XDF_IGNORE_WHITESPACE: u64 = 1 << 1;
pub const XDF_IGNORE_WHITESPACE_CHANGE: u64 = 1 << 2;
pub const XDF_IGNORE_WHITESPACE_AT_EOL: u64 = 1 << 3;
pub const XDF_IGNORE_CR_AT_EOL: u64 = 1 << 4;
pub const XDF_IGNORE_BLANK_LINES: u64 = 1 << 7;
pub const XDF_PATIENCE_DIFF: u64 = 1 << 14;
pub const XDF_HISTOGRAM_DIFF: u64 = 1 << 15;
pub const XDF_INDENT_HEURISTIC: u64 = 1 << 23;

/// Result of parsing the 'diffopt' option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffOptions {
    pub flags: u64,
    pub context: c_long,
    pub linematch_lines: c_int,
    pub foldcolumn: c_int,
    pub algorithm: u64,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            flags: DIFF_INTERNAL | DIFF_FILLER | DIFF_CLOSE_OFF,
            context: 6,
            linematch_lines: 0,
            foldcolumn: 2,
            algorithm: 0,
        }
    }
}

/// Parse the comma separated 'diffopt' value into [`DiffOptions`].
pub fn parse_diffopt(value: &str) -> Result<DiffOptions, String> {
    let mut opts = DiffOptions::default();
    for item in value.split(',').filter(|s| !s.is_empty()) {
        if item == "filler" {
            opts.flags |= DIFF_FILLER;
        } else if item == "anchor" {
            opts.flags |= DIFF_ANCHOR;
        } else if let Some(rest) = item.strip_prefix("context:") {
            opts.context = rest.parse().unwrap_or(opts.context);
        } else if item == "iblank" {
            opts.flags |= DIFF_IBLANK;
        } else if item == "icase" {
            opts.flags |= DIFF_ICASE;
        } else if item == "iwhiteall" {
            opts.flags |= DIFF_IWHITEALL;
        } else if item == "iwhiteeol" {
            opts.flags |= DIFF_IWHITEEOL;
        } else if item == "iwhite" {
            opts.flags |= DIFF_IWHITE;
        } else if item == "horizontal" {
            opts.flags |= DIFF_HORIZONTAL;
        } else if item == "vertical" {
            opts.flags |= DIFF_VERTICAL;
        } else if let Some(rest) = item.strip_prefix("foldcolumn:") {
            opts.foldcolumn = rest.parse().unwrap_or(opts.foldcolumn);
        } else if item == "hiddenoff" {
            opts.flags |= DIFF_HIDDEN_OFF;
        } else if item == "closeoff" {
            opts.flags |= DIFF_CLOSE_OFF;
        } else if item == "followwrap" {
            opts.flags |= DIFF_FOLLOWWRAP;
        } else if item == "indent-heuristic" {
            opts.algorithm |= XDF_INDENT_HEURISTIC;
        } else if item == "internal" {
            opts.flags |= DIFF_INTERNAL;
        } else if let Some(rest) = item.strip_prefix("algorithm:") {
            opts.algorithm |= match rest {
                "myers" => 0,
                "minimal" => XDF_NEED_MINIMAL,
                "patience" => XDF_PATIENCE_DIFF,
                "histogram" => XDF_HISTOGRAM_DIFF,
                _ => return Err(format!("unknown algorithm: {rest}")),
            };
        } else if let Some(rest) = item.strip_prefix("inline:") {
            opts.flags &= !ALL_INLINE;
            opts.flags |= match rest {
                "none" => DIFF_INLINE_NONE,
                "simple" => DIFF_INLINE_SIMPLE,
                "char" => DIFF_INLINE_CHAR,
                "word" => DIFF_INLINE_WORD,
                _ => return Err(format!("unknown inline mode: {rest}")),
            };
        } else if let Some(rest) = item.strip_prefix("linematch:") {
            opts.linematch_lines = rest.parse().unwrap_or(0);
            opts.flags |= DIFF_LINEMATCH | DIFF_FILLER;
        } else {
            return Err(format!("unknown diff option: {item}"));
        }
    }

    if (opts.flags & DIFF_HORIZONTAL != 0) && (opts.flags & DIFF_VERTICAL != 0) {
        return Err("cannot combine horizontal and vertical".into());
    }

    if opts.context == 0 {
        opts.context = 1;
    }

    Ok(opts)
}

/// Single hunk of a diff result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    pub lnum_orig: c_long,
    pub count_orig: c_long,
    pub lnum_new: c_long,
    pub count_new: c_long,
}

/// Compute the diff hunks for two strings using `rust_xdiff`.
pub fn calculate_diff(a: &str, b: &str, opts: &DiffOptions) -> Result<Vec<DiffHunk>, c_int> {
    let mf1 = mmfile_t {
        ptr: a.as_ptr() as *const c_char,
        size: a.len() as c_long,
    };
    let mf2 = mmfile_t {
        ptr: b.as_ptr() as *const c_char,
        size: b.len() as c_long,
    };

    let xpp = xpparam_t {
        flags: opts.algorithm,
        anchors: core::ptr::null_mut(),
        anchors_nr: 0,
    };
    let xecfg = xdemitconf_t {
        ctxlen: opts.context,
        interhunkctxlen: 0,
        flags: opts.flags,
        find_func: None,
        find_func_priv: core::ptr::null_mut(),
        hunk_func: None,
    };

    unsafe extern "C" fn collect(
        priv_: *mut c_void,
        start_a: c_long,
        count_a: c_long,
        start_b: c_long,
        count_b: c_long,
        _func: *const c_char,
        _size: c_long,
    ) -> c_int {
        let vec = &mut *(priv_ as *mut Vec<DiffHunk>);
        vec.push(DiffHunk {
            lnum_orig: start_a + 1,
            count_orig: count_a,
            lnum_new: start_b + 1,
            count_new: count_b,
        });
        0
    }

    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut ecb = xdemitcb_t {
        priv_: &mut hunks as *mut _ as *mut c_void,
        out_hunk: Some(collect),
        out_line: None,
    };

    let res = unsafe { rs_xdl_diff(&mf1, &mf2, &xpp, &xecfg, &mut ecb) };
    if res == 0 {
        Ok(hunks)
    } else {
        Err(res)
    }
}

pub fn diffopt_horizontal(opts: &DiffOptions) -> bool {
    opts.flags & DIFF_HORIZONTAL != 0
}

pub fn diffopt_hiddenoff(opts: &DiffOptions) -> bool {
    opts.flags & DIFF_HIDDEN_OFF != 0
}

pub fn diffopt_closeoff(opts: &DiffOptions) -> bool {
    opts.flags & DIFF_CLOSE_OFF != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() {
        let opts = parse_diffopt("filler,context:3,inline:none").unwrap();
        assert_eq!(opts.context, 3);
        assert!(opts.flags & DIFF_FILLER != 0);
        assert!(opts.flags & DIFF_INLINE_NONE != 0);
    }

    #[test]
    fn diff_simple() {
        let opts = DiffOptions::default();
        let hunks = calculate_diff("a\nb\n", "a\nc\n", &opts).unwrap();
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].lnum_orig, 2);
        assert_eq!(hunks[0].lnum_new, 2);
    }
}
