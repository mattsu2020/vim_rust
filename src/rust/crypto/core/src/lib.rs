use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};

use blowfish::cipher::{BlockDecrypt, BlockEncrypt, NewBlockCipher};
use blowfish::cipher::generic_array::GenericArray;
use blowfish::Blowfish;
use ring::digest::{digest, SHA256};

#[repr(C)]
pub struct cryptstate_T {
    pub method_nr: c_int,
    pub method_state: *mut c_void,
}

struct BlowfishState {
    cipher: Blowfish,
}

#[no_mangle]
pub extern "C" fn crypt_blowfish_init(state: *mut cryptstate_T, key: *const c_char, _arg: *mut c_void) -> c_int {
    if state.is_null() || key.is_null() {
        return 0;
    }
    let key_slice = unsafe { CStr::from_ptr(key).to_bytes() };
    let digest = digest(&SHA256, key_slice);
    let key_bytes = &digest.as_ref()[0..16];
    let cipher = Blowfish::new_from_slice(key_bytes).unwrap();
    let boxed = Box::new(BlowfishState { cipher });
    unsafe {
        (*state).method_state = Box::into_raw(boxed) as *mut c_void;
    }
    1
}

#[no_mangle]
pub extern "C" fn crypt_blowfish_encode(state: *mut cryptstate_T, from: *const u8, len: usize, to: *mut u8, _last: c_int) {
    if state.is_null() || from.is_null() || to.is_null() {
        return;
    }
    let st = unsafe { &mut *(*state).method_state.cast::<BlowfishState>() };
    let mut data = unsafe { std::slice::from_raw_parts(from, len).to_vec() };
    for chunk in data.chunks_mut(8) {
        if chunk.len() == 8 {
            let mut block = GenericArray::from_mut_slice(chunk);
            st.cipher.encrypt_block(&mut block);
        }
    }
    unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), to, len); }
}

#[no_mangle]
pub extern "C" fn crypt_blowfish_decode(state: *mut cryptstate_T, from: *const u8, len: usize, to: *mut u8, _last: c_int) {
    if state.is_null() || from.is_null() || to.is_null() {
        return;
    }
    let st = unsafe { &mut *(*state).method_state.cast::<BlowfishState>() };
    let mut data = unsafe { std::slice::from_raw_parts(from, len).to_vec() };
    for chunk in data.chunks_mut(8) {
        if chunk.len() == 8 {
            let mut block = GenericArray::from_mut_slice(chunk);
            st.cipher.decrypt_block(&mut block);
        }
    }
    unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), to, len); }
}

#[no_mangle]
pub extern "C" fn crypt_blowfish_encode_inplace(state: *mut cryptstate_T, buf: *mut u8, len: usize, _p2: *mut u8, last: c_int) {
    crypt_blowfish_encode(state, buf as *const u8, len, buf, last);
}

#[no_mangle]
pub extern "C" fn crypt_blowfish_decode_inplace(state: *mut cryptstate_T, buf: *mut u8, len: usize, _p2: *mut u8, last: c_int) {
    crypt_blowfish_decode(state, buf as *const u8, len, buf, last);
}

// Representation of cryptmethod_T from the C side
#[repr(C)]
pub struct cryptmethod_T {
    pub name: *const c_char,
    pub magic: *const c_char,
    pub salt_len: c_int,
    pub seed_len: c_int,
    pub add_len: c_int,
    pub works_inplace: c_int,
    pub whole_undofile: c_int,
    pub self_test_fn: Option<extern "C" fn() -> c_int>,
    pub init_fn: Option<extern "C" fn(*mut cryptstate_T, *const c_char, *mut c_void) -> c_int>,
    pub encode_fn: Option<extern "C" fn(*mut cryptstate_T, *const u8, usize, *mut u8, c_int)>,
    pub decode_fn: Option<extern "C" fn(*mut cryptstate_T, *const u8, usize, *mut u8, c_int)>,
    pub encode_buffer_fn: Option<extern "C" fn(*mut cryptstate_T, *const u8, usize, *mut *mut u8, c_int) -> i64>,
    pub decode_buffer_fn: Option<extern "C" fn(*mut cryptstate_T, *const u8, usize, *mut *mut u8, c_int) -> i64>,
    pub encode_inplace_fn: Option<extern "C" fn(*mut cryptstate_T, *mut u8, usize, *mut u8, c_int)>,
    pub decode_inplace_fn: Option<extern "C" fn(*mut cryptstate_T, *mut u8, usize, *mut u8, c_int)>,
}

unsafe impl Sync for cryptmethod_T {}

const ZIP_NAME: &[u8] = b"zip\0";
const ZIP_MAGIC: &[u8] = b"VimCrypt~01!\0";
const BF_NAME: &[u8] = b"blowfish\0";
const BF_MAGIC: &[u8] = b"VimCrypt~02!\0";
const BF2_NAME: &[u8] = b"blowfish2\0";
const BF2_MAGIC: &[u8] = b"VimCrypt~03!\0";
const XCHACHA_NAME: &[u8] = b"xchacha20\0";
const XCHACHA_MAGIC: &[u8] = b"VimCrypt~04!\0";
const XCHACHA2_NAME: &[u8] = b"xchacha20sodium\0";
const XCHACHA2_MAGIC: &[u8] = b"VimCrypt~05!\0";

static METHODS: [cryptmethod_T; 5] = [
    cryptmethod_T {
        name: ZIP_NAME.as_ptr() as *const c_char,
        magic: ZIP_MAGIC.as_ptr() as *const c_char,
        salt_len: 0,
        seed_len: 0,
        add_len: 0,
        works_inplace: 1,
        whole_undofile: 0,
        self_test_fn: None,
        init_fn: None,
        encode_fn: None,
        decode_fn: None,
        encode_buffer_fn: None,
        decode_buffer_fn: None,
        encode_inplace_fn: None,
        decode_inplace_fn: None,
    },
    cryptmethod_T {
        name: BF_NAME.as_ptr() as *const c_char,
        magic: BF_MAGIC.as_ptr() as *const c_char,
        salt_len: 8,
        seed_len: 8,
        add_len: 0,
        works_inplace: 1,
        whole_undofile: 0,
        self_test_fn: None,
        init_fn: Some(crypt_blowfish_init),
        encode_fn: Some(crypt_blowfish_encode),
        decode_fn: Some(crypt_blowfish_decode),
        encode_buffer_fn: None,
        decode_buffer_fn: None,
        encode_inplace_fn: Some(crypt_blowfish_encode_inplace),
        decode_inplace_fn: Some(crypt_blowfish_decode_inplace),
    },
    cryptmethod_T {
        name: BF2_NAME.as_ptr() as *const c_char,
        magic: BF2_MAGIC.as_ptr() as *const c_char,
        salt_len: 8,
        seed_len: 8,
        add_len: 0,
        works_inplace: 1,
        whole_undofile: 1,
        self_test_fn: None,
        init_fn: Some(crypt_blowfish_init),
        encode_fn: Some(crypt_blowfish_encode),
        decode_fn: Some(crypt_blowfish_decode),
        encode_buffer_fn: None,
        decode_buffer_fn: None,
        encode_inplace_fn: Some(crypt_blowfish_encode_inplace),
        decode_inplace_fn: Some(crypt_blowfish_decode_inplace),
    },
    cryptmethod_T {
        name: XCHACHA_NAME.as_ptr() as *const c_char,
        magic: XCHACHA_MAGIC.as_ptr() as *const c_char,
        salt_len: 16,
        seed_len: 8,
        add_len: 0,
        works_inplace: 0,
        whole_undofile: 0,
        self_test_fn: None,
        init_fn: None,
        encode_fn: None,
        decode_fn: None,
        encode_buffer_fn: None,
        decode_buffer_fn: None,
        encode_inplace_fn: None,
        decode_inplace_fn: None,
    },
    cryptmethod_T {
        name: XCHACHA2_NAME.as_ptr() as *const c_char,
        magic: XCHACHA2_MAGIC.as_ptr() as *const c_char,
        salt_len: 16,
        seed_len: 8,
        add_len: 0,
        works_inplace: 0,
        whole_undofile: 0,
        self_test_fn: None,
        init_fn: None,
        encode_fn: None,
        decode_fn: None,
        encode_buffer_fn: None,
        decode_buffer_fn: None,
        encode_inplace_fn: None,
        decode_inplace_fn: None,
    },
];

#[no_mangle]
pub extern "C" fn rust_crypt_methods() -> *mut cryptmethod_T {
    METHODS.as_ptr() as *mut cryptmethod_T
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blowfish_roundtrip() {
        let mut state = cryptstate_T { method_nr: 0, method_state: std::ptr::null_mut() };
        let key = std::ffi::CString::new("test").unwrap();
        assert_eq!(1, crypt_blowfish_init(&mut state, key.as_ptr(), std::ptr::null_mut()));
        let data = b"hello world";
        let mut enc = vec![0u8; data.len()];
        let mut dec = vec![0u8; data.len()];
        crypt_blowfish_encode(&mut state, data.as_ptr(), data.len(), enc.as_mut_ptr(), 1);
        crypt_blowfish_decode(&mut state, enc.as_ptr(), enc.len(), dec.as_mut_ptr(), 1);
        assert_eq!(data.to_vec(), dec);
    }
}
