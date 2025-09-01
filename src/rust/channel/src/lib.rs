use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::{io, ptr};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::runtime::Runtime;
use std::process::Stdio;

type Callback = extern "C" fn(*const c_char, usize, *mut c_void);

pub struct Channel {
    rt: Runtime,
    kind: ChannelKind,
}

enum ChannelKind {
    Tcp(Option<TcpStream>),
    Job {
        child: Child,
        stdin: Option<ChildStdin>,
        stdout: Option<ChildStdout>,
    },
}

#[no_mangle]
pub extern "C" fn channel_open(addr: *const c_char) -> *mut Channel {
    if addr.is_null() {
        return ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(addr) };
    let addr_str = match c_str.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    match rt.block_on(TcpStream::connect(addr_str)) {
        Ok(stream) => Box::into_raw(Box::new(Channel {
            rt,
            kind: ChannelKind::Tcp(Some(stream)),
        })),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn channel_spawn(cmd: *const c_char) -> *mut Channel {
    if cmd.is_null() {
        return ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(cmd) };
    let cmd_str = match c_str.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    let mut child = match Command::new(cmd_str)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return ptr::null_mut(),
    };
    let stdin = child.stdin.take();
    let stdout = child.stdout.take();
    Box::into_raw(Box::new(Channel {
        rt,
        kind: ChannelKind::Job { child, stdin, stdout },
    }))
}

#[no_mangle]
pub extern "C" fn channel_send(
    chan: *mut Channel,
    data: *const c_char,
    len: usize,
) -> c_int {
    if chan.is_null() || data.is_null() {
        return -1;
    }
    let chan = unsafe { &mut *chan };
    let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
    let res = match &mut chan.kind {
        ChannelKind::Tcp(stream) => {
            if let Some(s) = stream.as_mut() {
                chan.rt.block_on(s.write_all(slice))
            } else {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "stream closed"))
            }
        }
        ChannelKind::Job { stdin, .. } => {
            if let Some(inp) = stdin.as_mut() {
                chan.rt.block_on(inp.write_all(slice))
            } else {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "stdin closed"))
            }
        }
    };
    match res {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn channel_receive(
    chan: *mut Channel,
    buf: *mut c_char,
    len: usize,
) -> isize {
    if chan.is_null() || buf.is_null() {
        return -1;
    }
    let chan = unsafe { &mut *chan };
    let slice = unsafe { std::slice::from_raw_parts_mut(buf as *mut u8, len) };
    let res = match &mut chan.kind {
        ChannelKind::Tcp(stream) => {
            if let Some(s) = stream.as_mut() {
                chan.rt.block_on(s.read(slice))
            } else {
                Ok(0)
            }
        }
        ChannelKind::Job { stdout, .. } => {
            if let Some(out) = stdout.as_mut() {
                chan.rt.block_on(out.read(slice))
            } else {
                Ok(0)
            }
        }
    };
    match res {
        Ok(n) => n as isize,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn channel_set_callback(
    chan: *mut Channel,
    cb: Option<Callback>,
    user: *mut c_void,
) -> c_int {
    if chan.is_null() || cb.is_none() {
        return -1;
    }
    let chan = unsafe { &mut *chan };
    let cb = cb.unwrap();

    match &mut chan.kind {
        ChannelKind::Tcp(stream) => {
            if let Some(s) = stream.take() {
                let user_ptr = user as usize;
                chan.rt.spawn(async move {
                    let mut stream = s;
                    let mut buf = [0u8; 1024];
                    loop {
                        match stream.read(&mut buf).await {
                            Ok(0) | Err(_) => break,
                            Ok(n) => {
                                let user = user_ptr as *mut c_void;
                                cb(buf.as_ptr() as *const c_char, n, user)
                            }
                        }
                    }
                });
            } else {
                return -1;
            }
        }
        ChannelKind::Job { stdout, .. } => {
            if let Some(out) = stdout.take() {
                let user_ptr = user as usize;
                chan.rt.spawn(async move {
                    let mut out = out;
                    let mut buf = [0u8; 1024];
                    loop {
                        match out.read(&mut buf).await {
                            Ok(0) | Err(_) => break,
                            Ok(n) => {
                                let user = user_ptr as *mut c_void;
                                cb(buf.as_ptr() as *const c_char, n, user)
                            }
                        }
                    }
                });
            } else {
                return -1;
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn channel_job_wait(chan: *mut Channel) -> c_int {
    if chan.is_null() {
        return -1;
    }
    let chan = unsafe { &mut *chan };
    match &mut chan.kind {
        ChannelKind::Job { child, .. } => match chan.rt.block_on(child.wait()) {
            Ok(status) => status.code().unwrap_or(-1),
            Err(_) => -1,
        },
        _ => -1,
    }
}

#[no_mangle]
pub extern "C" fn channel_close_stdin(chan: *mut Channel) -> c_int {
    if chan.is_null() {
        return -1;
    }
    let chan = unsafe { &mut *chan };
    match &mut chan.kind {
        ChannelKind::Job { stdin, .. } => {
            stdin.take();
            0
        }
        _ => -1,
    }
}

#[no_mangle]
pub extern "C" fn channel_close(chan: *mut Channel) {
    if chan.is_null() {
        return;
    }
    unsafe {
        let mut boxed = Box::from_raw(chan);
        if let ChannelKind::Job { child, .. } = &mut boxed.kind {
            let _ = boxed.rt.block_on(child.kill());
        }
        // boxed dropped here
    }
}

