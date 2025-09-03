use crossterm::{style::Print, QueueableCommand};
use std::ffi::CStr;
use std::io::{stdout, Stdout, Write};
use std::os::raw::{c_char, c_int};

/// Generic UI object wrapping a writable target.
pub struct Ui<W: Write> {
    pub out: W,
}

impl<W: Write> Ui<W> {
    /// Create a new UI from the given writer.
    pub fn new(out: W) -> Self {
        Self { out }
    }

    /// Print text to the underlying writer using crossterm.
    pub fn print(&mut self, text: &str) -> std::io::Result<()> {
        self.out.queue(Print(text))?;
        self.out.flush()
    }
}

/// Concrete UI using the real standard output handle.
pub type TermUi = Ui<Stdout>;

#[no_mangle]
pub extern "C" fn rs_ui_new() -> *mut TermUi {
    Box::into_raw(Box::new(Ui::new(stdout())))
}

#[no_mangle]
pub unsafe extern "C" fn rs_ui_free(ptr: *mut TermUi) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

#[no_mangle]
pub unsafe extern "C" fn rs_ui_print(ptr: *mut TermUi, s: *const c_char) -> c_int {
    if ptr.is_null() || s.is_null() {
        return -1;
    }
    let ui = &mut *ptr;
    let c_str = CStr::from_ptr(s);
    match c_str.to_str() {
        Ok(text) => ui.print(text).map(|_| 0).unwrap_or(-1),
        Err(_) => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_to_buffer() {
        let mut ui = Ui::new(Vec::<u8>::new());
        assert_eq!(ui.print("hello").unwrap(), ());
        assert_eq!(ui.out, b"hello".to_vec());
    }
}
