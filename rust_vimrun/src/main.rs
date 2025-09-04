#![cfg(windows)]
use std::{ffi::c_void, ptr};
use std::vec::Vec;
use windows_sys::Win32::System::Console::{GetStdHandle, WriteConsoleW, STD_OUTPUT_HANDLE};
use windows_sys::Win32::System::Environment::GetCommandLineW;

extern "C" {
    fn _wsystem(cmd: *const u16) -> i32;
    fn _kbhit() -> i32;
    fn _getch() -> i32;
}

fn main() {
    unsafe {
        let mut p = GetCommandLineW();
        let mut inquote = false;
        while *p != 0 {
            if *p == b'"' as u16 {
                inquote = !inquote;
            } else if !inquote && *p == b' ' as u16 {
                p = p.add(1);
                break;
            }
            p = p.add(1);
        }
        while *p == b' ' as u16 {
            p = p.add(1);
        }

        let mut silent = false;
        if *p == b'-' as u16 && *p.add(1) == b's' as u16 && *p.add(2) == b' ' as u16 {
            silent = true;
            p = p.add(3);
            while *p == b' ' as u16 {
                p = p.add(1);
            }
        }

        let mut len = 0;
        let mut t = p;
        while *t != 0 {
            len += 1;
            t = t.add(1);
        }
        let slice = std::slice::from_raw_parts(p, len);

        let mut written = 0u32;
        let hstdout = GetStdHandle(STD_OUTPUT_HANDLE);
        WriteConsoleW(
            hstdout,
            slice.as_ptr() as *const c_void,
            slice.len() as u32,
            &mut written,
            ptr::null_mut(),
        );
        let newline: [u16; 2] = ['\r' as u16, '\n' as u16];
        WriteConsoleW(
            hstdout,
            newline.as_ptr() as *const c_void,
            2,
            &mut written,
            ptr::null_mut(),
        );

        let mut cmd_vec: Vec<u16>;
        let cmd_ptr = if len >= 2 && slice[0] == b'"' as u16 && slice[len - 1] == b'"' as u16 {
            cmd_vec = Vec::with_capacity(len + 3);
            cmd_vec.push('(' as u16);
            cmd_vec.extend_from_slice(slice);
            cmd_vec.push(')' as u16);
            cmd_vec.push(0);
            cmd_vec.as_ptr()
        } else {
            cmd_vec = slice.to_vec();
            cmd_vec.push(0);
            cmd_vec.as_ptr()
        };

        let retval = _wsystem(cmd_ptr);
        if retval == -1 {
            eprintln!("vimrun _wsystem() failed");
        } else if retval != 0 {
            println!("shell returned {}", retval);
        }

        if !silent {
            println!("Hit any key to close this window...");
            while _kbhit() != 0 {
                _getch();
            }
            _getch();
        }
        std::process::exit(retval);
    }
}
