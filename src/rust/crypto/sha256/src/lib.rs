use rand::RngCore;
use sha2::{Digest, Sha256};
use std::ffi::c_uchar;
use std::ptr;

#[repr(C)]
pub struct context_sha256_T {
    data: [u8; 104],
}

fn store_hasher(ctx: *mut context_sha256_T, hasher: *mut Sha256) {
    unsafe {
        ptr::write_unaligned(ctx as *mut *mut Sha256, hasher);
    }
}

fn take_hasher(ctx: *mut context_sha256_T) -> *mut Sha256 {
    unsafe { ptr::read_unaligned(ctx as *const *mut Sha256) }
}

const HEX: &[u8; 16] = b"0123456789abcdef";
static mut HEXIT: [u8; 65] = [0; 65];

#[no_mangle]
pub extern "C" fn sha256_start(ctx: *mut context_sha256_T) {
    let hasher = Box::new(Sha256::new());
    store_hasher(ctx, Box::into_raw(hasher));
}

#[no_mangle]
pub extern "C" fn sha256_update(
    ctx: *mut context_sha256_T,
    input: *const c_uchar,
    length: u32,
) {
    let hasher_ptr = take_hasher(ctx);
    if hasher_ptr.is_null() {
        return;
    }
    unsafe {
        let slice = std::slice::from_raw_parts(input, length as usize);
        (*hasher_ptr).update(slice);
        store_hasher(ctx, hasher_ptr);
    }
}

#[no_mangle]
pub extern "C" fn sha256_finish(ctx: *mut context_sha256_T, digest: *mut c_uchar) {
    let hasher_ptr = take_hasher(ctx);
    if hasher_ptr.is_null() {
        return;
    }
    let hash = unsafe { Box::from_raw(hasher_ptr).finalize() };
    unsafe {
        ptr::copy_nonoverlapping(hash.as_ptr(), digest, 32);
        store_hasher(ctx, ptr::null_mut());
    }
}

#[no_mangle]
pub extern "C" fn sha256_bytes(
    buf: *const c_uchar,
    buf_len: i32,
    salt: *const c_uchar,
    salt_len: i32,
) -> *const c_uchar {
    unsafe {
        let mut hasher = Sha256::new();
        let slice = std::slice::from_raw_parts(buf, buf_len as usize);
        hasher.update(slice);
        if !salt.is_null() {
            let sslice = std::slice::from_raw_parts(salt, salt_len as usize);
            hasher.update(sslice);
        }
        let result = hasher.finalize();
        for (i, b) in result.iter().enumerate() {
            HEXIT[i * 2] = HEX[(b >> 4) as usize];
            HEXIT[i * 2 + 1] = HEX[(b & 0x0f) as usize];
        }
        HEXIT[64] = 0;
        HEXIT.as_ptr()
    }
}

unsafe fn c_strlen(mut s: *const c_uchar) -> usize {
    let mut len = 0usize;
    while !s.is_null() && *s != 0 {
        len += 1;
        s = s.add(1);
    }
    len
}

#[no_mangle]
pub extern "C" fn sha256_key(
    buf: *const c_uchar,
    salt: *const c_uchar,
    salt_len: i32,
) -> *const c_uchar {
    unsafe {
        if buf.is_null() || *buf == 0 {
            return b"\0".as_ptr();
        }
        let len = c_strlen(buf);
        sha256_bytes(buf, len as i32, salt, salt_len)
    }
}

#[no_mangle]
pub extern "C" fn sha256_self_test() -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn sha2_seed(
    header: *mut c_uchar,
    header_len: i32,
    salt: *mut c_uchar,
    salt_len: i32,
) {
    let mut random_data = [0u8; 1000];
    rand::thread_rng().fill_bytes(&mut random_data);
    let mut hasher = Sha256::new();
    hasher.update(&random_data);
    let hash = hasher.finalize();
    unsafe {
        for i in 0..header_len as usize {
            *header.add(i) = hash[i % 32];
        }
        if !salt.is_null() {
            for i in 0..salt_len as usize {
                *salt.add(i) = hash[(i + header_len as usize) % 32];
            }
        }
    }
}
