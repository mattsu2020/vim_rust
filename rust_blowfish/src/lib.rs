use libc::{c_char, c_int, c_void};
use std::ffi::CStr;
use std::slice;

use blowfish::cipher::{generic_array::GenericArray, BlockEncrypt, NewBlockCipher};
use blowfish::Blowfish;
use hex::{decode, encode};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[repr(C)]
pub struct cryptstate_T {
    pub method_nr: c_int,
    pub method_state: *mut c_void,
}

#[repr(C)]
pub struct crypt_arg_T {
    pub cat_salt: *mut u8,
    pub cat_salt_len: c_int,
    pub cat_seed: *mut u8,
    pub cat_seed_len: c_int,
    pub cat_add: *mut u8,
    pub cat_add_len: c_int,
    pub cat_init_from_file: c_int,
}

const CRYPT_M_BF: c_int = 1;
const CRYPT_M_BF2: c_int = 2;

struct BlowfishState {
    cipher: Blowfish,
    cfb: Vec<u8>,
    cfb_len: usize,
    randbyte_offset: usize,
    update_offset: usize,
}

impl BlowfishState {
    fn randbyte(&mut self) -> u8 {
        if (self.randbyte_offset & 7) == 0 {
            let start = self.randbyte_offset;
            let mut block = GenericArray::clone_from_slice(&self.cfb[start..start + 8]);
            self.cipher.encrypt_block(&mut block);
            self.cfb[start..start + 8].copy_from_slice(&block);
        }
        let t = self.cfb[self.randbyte_offset];
        self.randbyte_offset += 1;
        if self.randbyte_offset == self.cfb_len {
            self.randbyte_offset = 0;
        }
        t
    }

    fn cfb_update(&mut self, c: u8) {
        self.cfb[self.update_offset] ^= c;
        self.update_offset += 1;
        if self.update_offset == self.cfb_len {
            self.update_offset = 0;
        }
    }
}

fn cfb_init(state: &mut BlowfishState, seed: &[u8]) {
    state.cfb.fill(0);
    state.randbyte_offset = 0;
    state.update_offset = 0;
    if !seed.is_empty() {
        let mi = std::cmp::max(seed.len(), state.cfb_len);
        for i in 0..mi {
            let idx = i % state.cfb_len;
            state.cfb[idx] ^= seed[i % seed.len()];
        }
    }
}

#[no_mangle]
pub extern "C" fn crypt_blowfish_init(
    state: *mut cryptstate_T,
    key: *const c_char,
    arg: *mut crypt_arg_T,
) -> c_int {
    if state.is_null() || key.is_null() || arg.is_null() {
        return 0;
    }
    let key_slice = unsafe { CStr::from_ptr(key).to_bytes() };
    let arg = unsafe { &*arg };
    let salt = unsafe { slice::from_raw_parts(arg.cat_salt, arg.cat_salt_len as usize) };
    let seed = unsafe { slice::from_raw_parts(arg.cat_seed, arg.cat_seed_len as usize) };
    let mut key_bytes = [0u8; 32];
    pbkdf2::<HmacSha256>(key_slice, salt, 1001, &mut key_bytes).expect("pbkdf2");
    let key_bytes = decode(encode(key_bytes)).unwrap();
    let cipher = match Blowfish::new_from_slice(&key_bytes) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    let cfb_len = if unsafe { (*state).method_nr } == CRYPT_M_BF {
        64
    } else {
        8
    };
    let mut st = BlowfishState {
        cipher,
        cfb: vec![0u8; cfb_len],
        cfb_len,
        randbyte_offset: 0,
        update_offset: 0,
    };
    cfb_init(&mut st, seed);
    unsafe {
        (*state).method_state = Box::into_raw(Box::new(st)) as *mut c_void;
    }
    1
}

#[no_mangle]
pub extern "C" fn crypt_blowfish_encode(
    state: *mut cryptstate_T,
    from: *const u8,
    len: usize,
    to: *mut u8,
    _last: c_int,
) {
    if state.is_null() || from.is_null() || to.is_null() {
        return;
    }
    let st = unsafe { &mut *(*state).method_state.cast::<BlowfishState>() };
    for i in 0..len {
        let z = unsafe { *from.add(i) };
        let t = st.randbyte();
        st.cfb_update(z);
        unsafe {
            *to.add(i) = t ^ z;
        }
    }
}

#[no_mangle]
pub extern "C" fn crypt_blowfish_decode(
    state: *mut cryptstate_T,
    from: *const u8,
    len: usize,
    to: *mut u8,
    _last: c_int,
) {
    if state.is_null() || from.is_null() || to.is_null() {
        return;
    }
    let st = unsafe { &mut *(*state).method_state.cast::<BlowfishState>() };
    for i in 0..len {
        let t = st.randbyte();
        let val = unsafe { *from.add(i) } ^ t;
        st.cfb_update(val);
        unsafe {
            *to.add(i) = val;
        }
    }
}

#[no_mangle]
pub extern "C" fn crypt_blowfish_encode_inplace(
    state: *mut cryptstate_T,
    buf: *mut u8,
    len: usize,
    _p2: *mut u8,
    last: c_int,
) {
    crypt_blowfish_encode(state, buf as *const u8, len, buf, last);
}

#[no_mangle]
pub extern "C" fn crypt_blowfish_decode_inplace(
    state: *mut cryptstate_T,
    buf: *mut u8,
    len: usize,
    _p2: *mut u8,
    last: c_int,
) {
    crypt_blowfish_decode(state, buf as *const u8, len, buf, last);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn roundtrip() {
        let key = CString::new("test").unwrap();
        let salt = b"12345678";
        let seed = b"abcdefgh";
        let mut arg = crypt_arg_T {
            cat_salt: salt.as_ptr() as *mut u8,
            cat_salt_len: salt.len() as c_int,
            cat_seed: seed.as_ptr() as *mut u8,
            cat_seed_len: seed.len() as c_int,
            cat_add: std::ptr::null_mut(),
            cat_add_len: 0,
            cat_init_from_file: 0,
        };
        let mut st_enc = cryptstate_T {
            method_nr: CRYPT_M_BF2,
            method_state: std::ptr::null_mut(),
        };
        assert_eq!(1, crypt_blowfish_init(&mut st_enc, key.as_ptr(), &mut arg));
        let data = b"hello world";
        let mut enc = vec![0u8; data.len()];
        crypt_blowfish_encode(&mut st_enc, data.as_ptr(), data.len(), enc.as_mut_ptr(), 1);
        let mut st_dec = cryptstate_T {
            method_nr: CRYPT_M_BF2,
            method_state: std::ptr::null_mut(),
        };
        assert_eq!(1, crypt_blowfish_init(&mut st_dec, key.as_ptr(), &mut arg));
        let mut dec = vec![0u8; data.len()];
        crypt_blowfish_decode(&mut st_dec, enc.as_ptr(), enc.len(), dec.as_mut_ptr(), 1);
        assert_eq!(data.to_vec(), dec);
    }
}
