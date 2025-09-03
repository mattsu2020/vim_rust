use libc::{c_char, c_int, size_t};
use std::ffi::{CStr, CString};
use std::io;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::*, Terminal as RatatuiTerminal};

pub struct Terminal {
    scrollback: Vec<CString>,
}

impl Terminal {
    pub fn new(_width: i32, _height: i32) -> io::Result<Self> {
        Ok(Self {
            scrollback: Vec::new(),
        })
    }

    pub fn write_input(&mut self, data: &[u8]) -> io::Result<()> {
        let line = String::from_utf8_lossy(data);
        self.record_line(&line);
        Ok(())
    }

    pub fn read_output(&mut self) -> io::Result<usize> {
        Ok(0)
    }

    pub fn record_line(&mut self, line: &str) {
        if let Ok(cstr) = CString::new(line) {
            self.scrollback.push(cstr);
        }
    }

    pub fn scrollback(&self) -> Vec<&CStr> {
        self.scrollback.iter().map(|s| s.as_c_str()).collect()
    }

    pub fn render(&self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal =
            RatatuiTerminal::new(backend).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        terminal
            .draw(|frame| {
                let block = Block::default().title("Scrollback").borders(Borders::ALL);
                let text = self
                    .scrollback
                    .iter()
                    .map(|s| s.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("\n");
                let paragraph = Paragraph::new(text).block(block);
                frame.render_widget(paragraph, frame.size());
            })
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        disable_raw_mode()?;
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn terminal_new(width: c_int, height: c_int) -> *mut Terminal {
    match Terminal::new(width, height) {
        Ok(t) => Box::into_raw(Box::new(t)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn terminal_free(term: *mut Terminal) {
    if !term.is_null() {
        unsafe {
            drop(Box::from_raw(term));
        }
    }
}

#[no_mangle]
pub extern "C" fn terminal_write_input(
    term: *mut Terminal,
    data: *const c_char,
    len: size_t,
) -> c_int {
    if term.is_null() || data.is_null() {
        return -1;
    }
    let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len as usize) };
    match unsafe { &mut *term }.write_input(slice) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn terminal_read_output(term: *mut Terminal) -> c_int {
    if term.is_null() {
        return -1;
    }
    match unsafe { &mut *term }.read_output() {
        Ok(sz) => sz as c_int,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn terminal_scrollback_len(term: *mut Terminal) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { (&*term).scrollback.len() as c_int }
}

#[no_mangle]
pub extern "C" fn terminal_scrollback_line(term: *mut Terminal, idx: c_int) -> *const c_char {
    if term.is_null() || idx < 0 {
        return std::ptr::null();
    }
    unsafe {
        let term = &*term;
        term.scrollback
            .get(idx as usize)
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrollback_records_lines() {
        let mut term = Terminal::new(80, 24).expect("new terminal");
        term.record_line("hello");
        let sb = term.scrollback();
        assert_eq!(sb.len(), 1);
        assert_eq!(sb[0].to_str().unwrap(), "hello");
    }
}
