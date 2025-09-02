use std::os::raw::c_int;

#[repr(C)]
#[derive(Default)]
pub struct Buffer {
    pub did_warn: bool,
    pub changed: bool,
    pub read_only: bool,
}

impl Buffer {
    pub fn new(read_only: bool) -> Self {
        Self { read_only, ..Default::default() }
    }
}

#[no_mangle]
pub extern "C" fn change_warning(buf: *mut Buffer) -> c_int {
    let b = unsafe { &mut *buf };
    if b.did_warn || b.changed || !b.read_only {
        return 0;
    }
    b.did_warn = true;
    1
}

#[no_mangle]
pub extern "C" fn changed(buf: *mut Buffer) {
    let b = unsafe { &mut *buf };
    b.changed = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warns_only_once() {
        let mut b = Buffer::new(true);
        let ptr = &mut b as *mut Buffer;
        assert_eq!(unsafe { change_warning(ptr) }, 1);
        assert_eq!(unsafe { change_warning(ptr) }, 0);
        assert!(b.did_warn);
    }
}
