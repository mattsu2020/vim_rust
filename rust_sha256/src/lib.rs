use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

const HEX: &[u8; 16] = b"0123456789abcdef";

/// Compute SHA-256 digest of arbitrary data and return a 32-byte array.
pub fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let result = Sha256::digest(data);
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

#[repr(C)]
pub struct context_sha256_T {
    hasher: *mut Sha256,
}

#[no_mangle]
pub extern "C" fn sha256_start(ctx: *mut context_sha256_T) {
    if ctx.is_null() {
        return;
    }
    let hasher = Box::new(Sha256::new());
    unsafe {
        (*ctx).hasher = Box::into_raw(hasher);
    }
}

#[no_mangle]
pub extern "C" fn sha256_update(ctx: *mut context_sha256_T, input: *const u8, length: u32) {
    if ctx.is_null() || input.is_null() {
        return;
    }
    unsafe {
        let hasher = &mut *(*ctx).hasher;
        let data = std::slice::from_raw_parts(input, length as usize);
        hasher.update(data);
    }
}

#[no_mangle]
pub extern "C" fn sha256_finish(ctx: *mut context_sha256_T, digest: *mut u8) {
    if ctx.is_null() || digest.is_null() {
        return;
    }
    unsafe {
        if (*ctx).hasher.is_null() {
            return;
        }
        let hasher = Box::from_raw((*ctx).hasher);
        let result = hasher.finalize();
        ptr::copy_nonoverlapping(result.as_ptr(), digest, 32);
        (*ctx).hasher = ptr::null_mut();
    }
}

static mut HEXIT: [u8; 65] = [0; 65];

fn hex_encode(bytes: [u8; 32]) -> *mut u8 {
    unsafe {
        for i in 0..32 {
            HEXIT[2 * i] = HEX[(bytes[i] >> 4) as usize];
            HEXIT[2 * i + 1] = HEX[(bytes[i] & 0x0F) as usize];
        }
        HEXIT[64] = 0;
        HEXIT.as_mut_ptr()
    }
}

#[no_mangle]
pub extern "C" fn sha256_bytes(
    buf: *const u8,
    buf_len: c_int,
    salt: *const u8,
    salt_len: c_int,
) -> *mut u8 {
    if buf.is_null() {
        return ptr::null_mut();
    }
    let mut data = unsafe { std::slice::from_raw_parts(buf, buf_len as usize).to_vec() };
    if !salt.is_null() && salt_len > 0 {
        let salt_slice = unsafe { std::slice::from_raw_parts(salt, salt_len as usize) };
        data.extend_from_slice(salt_slice);
    }
    let digest = sha256_digest(&data);
    hex_encode(digest)
}

#[no_mangle]
pub extern "C" fn sha256_key(
    buf: *const u8,
    salt: *const u8,
    salt_len: c_int,
) -> *mut u8 {
    if buf.is_null() {
        return b"\0".as_ptr() as *mut u8;
    }
    unsafe {
        if *buf == 0 {
            return b"\0".as_ptr() as *mut u8;
        }
        let len = CStr::from_ptr(buf as *const c_char).to_bytes().len() as c_int;
        sha256_bytes(buf, len, salt, salt_len)
    }
}

#[no_mangle]
pub extern "C" fn sha256_self_test() -> c_int {
    let tests = [
        (
            b"abc".as_ref(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".as_ref(),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
    ];
    for (msg, expect) in tests {
        let digest = Sha256::digest(msg);
        let mut hex = [0u8; 64];
        for i in 0..32 {
            hex[2 * i] = HEX[(digest[i] >> 4) as usize];
            hex[2 * i + 1] = HEX[(digest[i] & 0x0F) as usize];
        }
        if hex != expect.as_bytes() {
            return 1;
        }
    }
    let mut hasher = Sha256::new();
    let buf = [b'a'; 1000];
    for _ in 0..1000 {
        hasher.update(&buf);
    }
    let digest = hasher.finalize();
    let mut hex = [0u8; 64];
    for i in 0..32 {
        hex[2 * i] = HEX[(digest[i] >> 4) as usize];
        hex[2 * i + 1] = HEX[(digest[i] & 0x0F) as usize];
    }
    if hex != *b"cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0" {
        return 1;
    }
    0
}

fn get_some_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64
}

#[no_mangle]
pub extern "C" fn sha2_seed(
    header: *mut u8,
    header_len: c_int,
    salt: *mut u8,
    salt_len: c_int,
) {
    let mut time_bytes = get_some_time().to_le_bytes().to_vec();
    time_bytes.resize(1000, 0);
    let digest = Sha256::digest(&time_bytes);
    unsafe {
        for i in 0..header_len as usize {
            *header.add(i) = digest[i % digest.len()];
        }
        if !salt.is_null() {
            for i in 0..salt_len as usize {
                *salt.add(i) = digest[(i + header_len as usize) % digest.len()];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_abc() {
        let digest = sha256_digest(b"abc");
        let hex: String = digest.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(hex, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
}
