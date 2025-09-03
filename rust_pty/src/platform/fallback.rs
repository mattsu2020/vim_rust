use std::io;

pub struct OpenptyResult;

pub fn open() -> io::Result<OpenptyResult> {
    Err(io::Error::new(io::ErrorKind::Other, "unsupported platform"))
}

pub async fn echo_loop(_fd: i32) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Other, "unsupported platform"))
}
