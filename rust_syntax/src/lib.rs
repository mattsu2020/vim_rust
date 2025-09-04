use std::ffi::{c_void, CStr};
use std::os::raw::{c_char, c_short};
use std::sync::Mutex;
use rust_highlight::register_rule;

/// State tracking for syntax highlighting.
///
/// The C implementation stores similar information in a set of globals such as
/// `current_lnum`, `current_col` and `current_finished` to keep track of where
/// parsing happens within a buffer【F:src/syntax.c†L269-L280】.  The Rust version
/// mirrors the relevant parts so that `syntax_start()` and `syn_update_ends()`
/// can be implemented on the Rust side while exposing a C-compatible API.
#[derive(Clone, Copy, Debug, Default)]
pub struct Lpos {
    pub lnum: i64,
    pub col: i32,
}

#[derive(Clone, Debug)]
pub struct SyntaxRule {
    pub id: i32,
    pub pattern: String,
}

#[derive(Clone, Debug)]
pub struct StateItem {
    pub id: i32,
    pub m_end: Lpos,
    pub h_start: Lpos,
    pub h_end: Lpos,
    pub ends: bool,
    pub flags: i64,
}

#[derive(Clone, Debug, Default)]
pub struct SyntaxState {
    /// Window pointer provided by the C caller; opaque to Rust.
    pub window: *mut c_void,
    /// Opaque pointer to the buffer currently being highlighted.
    pub buffer: *mut c_void,
    /// Opaque pointer to the syntax block.
    pub block: *mut c_void,
    /// Current line number being parsed.
    pub lnum: i64,
    /// Current column within the line.
    pub col: i32,
    /// Whether the current state was stored for reuse.
    pub state_stored: bool,
    /// Flag indicating that the current line has been finished.
    pub finished: bool,
    /// Next group list pointer, mirroring `current_next_list` in C.
    pub next_list: *mut c_short,
    /// Flags for the next group list.
    pub next_flags: i32,
    /// Registered matching rules.
    pub rules: Vec<SyntaxRule>,
    /// Active state items after rule evaluation.
    pub stack: Vec<StateItem>,
}

// Raw pointers are opaque handles; sharing them across threads is safe as they
// are never dereferenced in Rust.
unsafe impl Send for SyntaxState {}
unsafe impl Sync for SyntaxState {}

/// Global syntax state shared with the C side.
static SYNTAX_STATE: Mutex<SyntaxState> = Mutex::new(SyntaxState {
    window: std::ptr::null_mut(),
    buffer: std::ptr::null_mut(),
    block: std::ptr::null_mut(),
    lnum: 0,
    col: 0,
    state_stored: false,
    finished: false,
    next_list: std::ptr::null_mut(),
    next_flags: 0,
    rules: Vec::new(),
    stack: Vec::new(),
});

/// Start syntax parsing for line `lnum` in window `wp`.
#[no_mangle]
pub extern "C" fn rs_syntax_start(wp: *mut c_void, lnum: i64) {
    let mut state = SYNTAX_STATE.lock().unwrap();
    state.window = wp;
    state.buffer = std::ptr::null_mut();
    state.block = std::ptr::null_mut();
    state.lnum = lnum;
    state.col = 0; // reset column as the C code does in syntax_start()
    state.state_stored = false;
    state.finished = false;
    state.next_list = std::ptr::null_mut();
    state.next_flags = 0;
    state.stack.clear();
}

/// Update the parser position.  When `startofline` is non-zero the parser moves
/// to the beginning of the next line; otherwise the column advances by one.
#[no_mangle]
pub extern "C" fn rs_syn_update(startofline: i32) {
    let mut state = SYNTAX_STATE.lock().unwrap();
    if startofline != 0 {
        state.lnum += 1;
        state.col = 0;
        state.finished = true;
        state.state_stored = false;
        state.next_list = std::ptr::null_mut();
        state.next_flags = 0;
    } else {
        state.col += 1;
        state.finished = false;
    }
}

/// Register a simple match rule with ID `id` and textual `pattern`.
#[no_mangle]
pub extern "C" fn rs_add_rule(id: i32, pattern: *const c_char) {
    let cstr = unsafe { CStr::from_ptr(pattern) };
    if let Ok(pat) = cstr.to_str() {
        let mut state = SYNTAX_STATE.lock().unwrap();
        state.rules.push(SyntaxRule { id, pattern: pat.to_string() });
        register_rule(id, pat);
    }
}

/// Clear all registered rules and active state items.
#[no_mangle]
pub extern "C" fn rs_clear_rules() {
    let mut state = SYNTAX_STATE.lock().unwrap();
    state.rules.clear();
    state.stack.clear();
}

/// Evaluate registered rules against `line`.  Returns the ID of the first
/// matching rule or 0 when no rule matches.
#[no_mangle]
pub extern "C" fn rs_eval_line(line: *const c_char) -> i32 {
    let cstr = unsafe { CStr::from_ptr(line) };
    let line = match cstr.to_str() {
        Ok(l) => l,
        Err(_) => return 0,
    };
    let mut state = SYNTAX_STATE.lock().unwrap();
    let lnum = state.lnum;
    let rules = state.rules.clone();
    for rule in rules {
        if let Some(pos) = line.find(&rule.pattern) {
            let start = pos as i32;
            let end = start + rule.pattern.len() as i32;
            state.stack.push(StateItem {
                id: rule.id,
                m_end: Lpos { lnum, col: end },
                h_start: Lpos { lnum, col: start },
                h_end: Lpos { lnum, col: end },
                ends: true,
                flags: 0,
            });
            state.col = end;
            return rule.id;
        }
    }
    0
}

/// Helper used by unit tests to inspect the current state.
fn get_state() -> SyntaxState {
    SYNTAX_STATE.lock().unwrap().clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_and_update_progression() {
        rs_clear_rules();
        rs_syntax_start(std::ptr::null_mut(), 10);
        let s = get_state();
        assert_eq!(s.lnum, 10);
        assert_eq!(s.col, 0);

        rs_syn_update(0); // advance within line
        let s = get_state();
        assert_eq!(s.lnum, 10);
        assert_eq!(s.col, 1);

        rs_syn_update(1); // move to next line
        let s = get_state();
        assert_eq!(s.lnum, 11);
        assert_eq!(s.col, 0);
        assert!(s.finished);
    }

    #[test]
    fn finished_flag() {
        rs_clear_rules();
        rs_syntax_start(std::ptr::null_mut(), 1);
        assert!(!get_state().finished);
        rs_syn_update(1);
        let s = get_state();
        assert!(s.finished);
    }

    #[test]
    fn regression_major_syntax_rules() {
        use std::ffi::CString;

        rs_clear_rules();
        let c_kw = CString::new("int").unwrap();
        rs_add_rule(1, c_kw.as_ptr());
        let rust_kw = CString::new("fn").unwrap();
        rs_add_rule(2, rust_kw.as_ptr());
        let py_kw = CString::new("def").unwrap();
        rs_add_rule(3, py_kw.as_ptr());

        rs_syntax_start(std::ptr::null_mut(), 1);

        let c_line = CString::new("int main() { return 0; }").unwrap();
        assert_eq!(rs_eval_line(c_line.as_ptr()), 1);
        rs_syn_update(1);

        let rust_line = CString::new("fn main() {}").unwrap();
        assert_eq!(rs_eval_line(rust_line.as_ptr()), 2);
        rs_syn_update(1);

        let py_line = CString::new("def main(): pass").unwrap();
        assert_eq!(rs_eval_line(py_line.as_ptr()), 3);
    }
}
