
#[repr(C)]
pub struct MemFile {
    data: Vec<u8>,
}

impl MemFile {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    pub fn write(&mut self, buf: &[u8]) {
        self.data.extend_from_slice(buf);
    }
    pub fn read(&self) -> &[u8] {
        &self.data
    }
}

#[no_mangle]
pub extern "C" fn rs_memfile_new() -> *mut MemFile {
    Box::into_raw(Box::new(MemFile::new()))
}

#[no_mangle]
pub extern "C" fn rs_memfile_write(ptr: *mut MemFile, data: *const u8, len: usize) {
    if ptr.is_null() || data.is_null() {
        return;
    }
    let mf = unsafe { &mut *ptr };
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    mf.write(slice);
}

#[no_mangle]
pub extern "C" fn rs_memfile_read(ptr: *const MemFile, len: *mut usize) -> *const u8 {
    if ptr.is_null() {
        return std::ptr::null();
    }
    let mf = unsafe { &*ptr };
    if !len.is_null() {
        unsafe { *len = mf.data.len(); }
    }
    mf.read().as_ptr()
}

#[no_mangle]
pub extern "C" fn rs_memfile_free(ptr: *mut MemFile) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr)); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mf = rs_memfile_new();
        let data = b"hello";
        rs_memfile_write(mf, data.as_ptr(), data.len());
        let mut len: usize = 0;
        let ptr = rs_memfile_read(mf, &mut len as *mut usize);
        assert_eq!(len, 5);
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        assert_eq!(slice, data);
        rs_memfile_free(mf);
    }
}
