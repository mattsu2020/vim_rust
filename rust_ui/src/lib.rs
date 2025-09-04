pub use rust_screen::ScreenBuffer;
use libc::{c_char, c_int};
use std::collections::VecDeque;
use std::ffi::CStr;
use std::sync::{LazyLock, Mutex};

static INPUT_BUF: LazyLock<Mutex<VecDeque<u8>>> = LazyLock::new(|| Mutex::new(VecDeque::new()));
static OUTPUT_BUF: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static GLOBAL_UI: LazyLock<Mutex<Option<Ui<CliRenderer>>>> =
    LazyLock::new(|| Mutex::new(None));

/// Write UI text directly from Rust, bypassing any C shim.
#[no_mangle]
pub unsafe extern "C" fn ui_write(msg: *const c_char, len: c_int) {
    if msg.is_null() || len <= 0 {
        return;
    }
    let slice = std::slice::from_raw_parts(msg as *const u8, len as usize);
    if let Ok(text) = std::str::from_utf8(slice) {
        OUTPUT_BUF.lock().unwrap().push(text.to_string());
    }
}

/// Simplified input reader backed by an internal buffer.
pub fn ui_inchar(buf: &mut [u8]) -> usize {
    let mut input = INPUT_BUF.lock().unwrap();
    let n = buf.len().min(input.len());
    for i in 0..n {
        buf[i] = input.pop_front().unwrap();
    }
    n
}

/// Push characters back to the start of the input buffer.
pub fn ui_inchar_undo(data: &[u8]) {
    let mut input = INPUT_BUF.lock().unwrap();
    for &b in data.iter().rev() {
        input.push_front(b);
    }
}

/// Helper for tests: append data to the input buffer.
pub fn push_input(data: &str) {
    INPUT_BUF.lock().unwrap().extend(data.as_bytes());
}

/// Helper for tests: take all pending output.
pub fn take_output() -> Vec<String> {
    let mut out = OUTPUT_BUF.lock().unwrap();
    let res = out.clone();
    out.clear();
    res
}

/// Initialize the global [`Ui`] instance used by C bindings.
pub fn init(width: usize, height: usize) {
    let mut ui = GLOBAL_UI.lock().unwrap();
    *ui = Some(Ui::new(width, height, CliRenderer));
}

/// Execute a closure with mutable access to the global [`Ui`] if initialized.
pub fn with_ui_mut<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut Ui<CliRenderer>) -> R,
{
    let mut ui = GLOBAL_UI.lock().unwrap();
    ui.as_mut().map(f)
}

/// Flush pending screen updates using the global [`Ui`].
pub fn flush() {
    let _ = with_ui_mut(|ui| ui.flush());
}

#[no_mangle]
pub extern "C" fn rs_ui_init(width: c_int, height: c_int) {
    init(width as usize, height as usize);
}

#[no_mangle]
pub extern "C" fn rs_ui_draw_line(row: c_int, text: *const c_char, attr: u8) {
    if text.is_null() {
        return;
    }
    let c_str = unsafe { CStr::from_ptr(text) };
    if let Ok(s) = c_str.to_str() {
        let _ = with_ui_mut(|ui| {
            ui.clear_line(row as usize, attr);
            ui.draw_text(row as usize, 0, s, attr);
        });
    }
}

#[no_mangle]
pub extern "C" fn rs_ui_flush() {
    flush();
}

/// Trait implemented by rendering backends.
pub trait Renderer {
    fn draw_line(&mut self, row: usize, text: &str, attrs: &[u8]);
}

/// Renderer that prints to stdout, useful for CLI mode.
pub struct CliRenderer;

impl Renderer for CliRenderer {
    fn draw_line(&mut self, row: usize, text: &str, _attrs: &[u8]) {
        println!("{row}: {text}");
    }
}

/// High level UI abstraction over [`ScreenBuffer`].
pub struct Ui<R: Renderer> {
    screen: ScreenBuffer,
    pub renderer: R,
}

impl<R: Renderer> Ui<R> {
    pub fn new(width: usize, height: usize, renderer: R) -> Self {
        Self {
            screen: ScreenBuffer::new(width, height),
            renderer,
        }
    }

    pub fn draw_text(&mut self, row: usize, col: usize, text: &str, attr: u8) {
        self.screen.draw_text(row, col, text, attr);
    }

    /// Clear a whole line with the given attribute.
    pub fn clear_line(&mut self, row: usize, attr: u8) {
        self.screen.clear_line(row, attr);
    }

    pub fn highlight(&mut self, row: usize, col: usize, len: usize, attr: u8) {
        self.screen.highlight_range(row, col, len, attr);
    }

    pub fn format_text(&self, text: &str, width: usize) -> String {
        ScreenBuffer::format_text(text, width)
    }

    pub fn flush(&mut self) {
        for diff in self.screen.flush_diff() {
            self.renderer.draw_line(diff.row, &diff.text, &diff.attrs);
        }
    }

    /// Return a copy of a line, used for tests.
    pub fn line(&self, row: usize) -> String {
        self.screen.line_as_string(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    struct CollectRenderer(pub Vec<String>);

    impl Renderer for CollectRenderer {
        fn draw_line(&mut self, _row: usize, text: &str, _attrs: &[u8]) {
            self.0.push(text.to_string());
        }
    }

    #[test]
    fn cli_and_gui_render() {
        let mut ui_cli = Ui::new(10, 2, CollectRenderer(Vec::new()));
        ui_cli.draw_text(0, 0, "hi", 1);
        ui_cli.flush();
        assert_eq!(ui_cli.renderer.0[0].trim_end(), "hi");

        let mut ui_gui = Ui::new(5, 1, CollectRenderer(Vec::new()));
        ui_gui.draw_text(0, 0, "ab", 1);
        ui_gui.highlight(0, 0, 2, 2);
        ui_gui.flush();
        assert_eq!(ui_gui.renderer.0[0], "ab   ");
    }

    #[test]
    fn formatting() {
        let ui = Ui::new(5, 1, CollectRenderer(Vec::new()));
        assert_eq!(ui.format_text("abc", 5), "abc  ");
        assert_eq!(ui.format_text("abcdef", 3), "abc");
    }

    #[test]
    fn input_buffer_roundtrip() {
        push_input("ab");
        let mut buf = [0u8; 2];
        assert_eq!(ui_inchar(&mut buf), 2);
        assert_eq!(buf, [b'a', b'b']);
        ui_inchar_undo(&buf);
        let mut buf2 = [0u8; 2];
        assert_eq!(ui_inchar(&mut buf2), 2);
        assert_eq!(buf2, [b'a', b'b']);
    }

    #[test]
    fn ui_write_captures_output() {
        unsafe {
            let msg = CString::new("hello").unwrap();
            ui_write(msg.as_ptr(), 5);
        }
        let out = take_output();
        assert_eq!(out, vec!["hello".to_string()]);
    }
}
