#[cfg(not(windows))]
fn main() {}

#[cfg(windows)]
fn main() {
    use std::ffi::OsString;
    use std::os::raw::c_int;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::System::Environment::GetCommandLineW;

    unsafe extern "C" {
        fn _wsystem(cmd: *const u16) -> c_int;
        fn _kbhit() -> c_int;
        fn _getch() -> c_int;
    }

    unsafe {
        let mut p = GetCommandLineW().0;
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

        let cmdlen = wcslen(p);
        let slice = std::slice::from_raw_parts(p, cmdlen);
        let cmd_str = OsString::from_wide(slice).to_string_lossy().to_string();
        use std::io::Write as _;
        let mut stdout = std::io::stdout();
        let _ = stdout.write_all(cmd_str.as_bytes());
        let _ = stdout.write_all(b"\r\n");

        let mut cmd_buf: Vec<u16> = Vec::new();
        let mut cmd_ptr = p;
        if cmdlen >= 2 && *p == b'"' as u16 && *p.add(cmdlen - 1) == b'"' as u16 {
            cmd_buf.reserve(cmdlen + 3);
            cmd_buf.push('(' as u16);
            for i in 0..cmdlen {
                cmd_buf.push(*p.add(i));
            }
            cmd_buf.push(')' as u16);
            cmd_buf.push(0);
            cmd_ptr = cmd_buf.as_ptr();
        }

        let retval = _wsystem(cmd_ptr);
        if retval == -1 {
            eprintln!("vimrun system(): {}", std::io::Error::last_os_error());
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
    }
}

#[cfg(windows)]
unsafe fn wcslen(mut p: *const u16) -> usize {
    let mut len = 0usize;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }
    len
}
