mod vterm {
    use libc::{c_char, c_int, size_t};

    #[repr(C)]
    pub struct VTerm {
        _private: [u8; 0],
    }

    extern "C" {
        pub fn vterm_new(cols: c_int, rows: c_int) -> *mut VTerm;
        pub fn vterm_free(vt: *mut VTerm);
        pub fn vterm_input_write(vt: *mut VTerm, data: *const c_char, len: size_t) -> size_t;
    }
}

use libc::{c_char, c_int, size_t};
use std::ffi::{CStr, CString};
use vterm::{vterm_free, vterm_input_write, vterm_new, VTerm};

#[cfg(unix)]
mod pty {
    use std::fs::File;
    use std::io;
    use std::os::unix::io::{FromRawFd, IntoRawFd};

    use nix::pty::openpty;
    use nix::pty::OpenptyResult;

    pub struct Pty {
        pub master: File,
        pub slave: File,
    }

    impl Pty {
        pub fn new() -> io::Result<Self> {
            let OpenptyResult { master, slave } = openpty(None, None)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let master_fd = master.into_raw_fd();
            let slave_fd = slave.into_raw_fd();
            unsafe {
                Ok(Self {
                    master: File::from_raw_fd(master_fd),
                    slave: File::from_raw_fd(slave_fd),
                })
            }
        }
    }
}

#[cfg(windows)]
mod pty {
    use std::io;

    pub struct Pty;

    impl Pty {
        pub fn new() -> io::Result<Self> {
            Err(io::Error::new(io::ErrorKind::Other, "PTY not supported"))
        }
    }
}

use pty::Pty;

pub struct Terminal {
    vterm: *mut VTerm,
    buffer: Vec<u8>,
    scrollback: Vec<CString>,
    pty: Pty,
}

impl Terminal {
    pub fn new(width: i32, height: i32) -> std::io::Result<Self> {
        let pty = Pty::new()?;
        let vterm = unsafe { vterm_new(width, height) };
        Ok(Self { vterm, buffer: Vec::new(), scrollback: Vec::new(), pty })
    }

    pub fn write_input(&mut self, data: &[u8]) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            use nix::unistd::write;
            use std::os::unix::io::AsRawFd;
            write(self.pty.master.as_raw_fd(), data)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
        #[cfg(windows)]
        {
            let _ = data;
        }
        Ok(())
    }

    pub fn read_output(&mut self) -> std::io::Result<usize> {
        let mut buf = [0u8; 1024];
        #[cfg(unix)]
        {
            use nix::unistd::read;
            use std::os::unix::io::AsRawFd;
            let size = read(self.pty.master.as_raw_fd(), &mut buf)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            self.buffer.extend_from_slice(&buf[..size]);
            unsafe {
                vterm_input_write(self.vterm, buf.as_ptr() as *const i8, size as usize);
            }
            Ok(size as usize)
        }
        #[cfg(windows)]
        {
            Ok(0)
        }
    }

    pub fn record_line(&mut self, line: &str) {
        if let Ok(cstr) = CString::new(line) {
            self.scrollback.push(cstr);
        }
    }

    pub fn scrollback(&self) -> Vec<&CStr> {
        self.scrollback.iter().map(|s| s.as_c_str()).collect()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        unsafe { vterm_free(self.vterm) };
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
        unsafe { drop(Box::from_raw(term)); }
    }
}

#[no_mangle]
pub extern "C" fn terminal_write_input(term: *mut Terminal, data: *const c_char, len: size_t) -> c_int {
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
