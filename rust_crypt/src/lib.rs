#[repr(C)]
pub struct cryptmethod_S {
    _private: [u8; 0],
}

#[no_mangle]
pub extern "C" fn rust_crypt_methods() -> *const cryptmethod_S {
    std::ptr::null()
}
