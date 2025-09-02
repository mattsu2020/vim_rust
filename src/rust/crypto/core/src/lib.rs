use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::slice;

use blowfish::cipher::{generic_array::GenericArray, BlockEncrypt, NewBlockCipher};
use blowfish::Blowfish;
use ring::digest::{Context, SHA256};

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
    cfb: [u8; 64],
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

fn sha256_hex(data: &[u8], salt: &[u8]) -> String {
    let mut ctx = Context::new(&SHA256);
    ctx.update(data);
    if !salt.is_empty() {
        ctx.update(salt);
    }
    let digest = ctx.finish();
    let mut s = String::with_capacity(64);
    for b in digest.as_ref() {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn hex_to_bytes(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let byte = u8::from_str_radix(&s[i..i + 2], 16).unwrap();
        out.push(byte);
    }
    out
}

fn derive_key(password: &[u8], salt: &[u8]) -> Vec<u8> {
    let mut key = sha256_hex(password, salt);
    for _ in 0..1000 {
        key = sha256_hex(key.as_bytes(), salt);
    }
    hex_to_bytes(&key)
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

    let key_bytes = derive_key(key_slice, salt);
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
        cfb: [0u8; 64],
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
        let ztemp = unsafe { *from.add(i) };
        let t = st.randbyte();
        st.cfb_update(ztemp);
        unsafe { *to.add(i) = t ^ ztemp };
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
        unsafe { *to.add(i) = val };
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
    pub init_fn:
        Option<extern "C" fn(*mut cryptstate_T, *const c_char, *mut crypt_arg_T) -> c_int>,
    pub encode_fn: Option<extern "C" fn(*mut cryptstate_T, *const u8, usize, *mut u8, c_int)>,
    pub decode_fn: Option<extern "C" fn(*mut cryptstate_T, *const u8, usize, *mut u8, c_int)>,
    pub encode_buffer_fn:
        Option<extern "C" fn(*mut cryptstate_T, *const u8, usize, *mut *mut u8, c_int) -> i64>,
    pub decode_buffer_fn:
        Option<extern "C" fn(*mut cryptstate_T, *const u8, usize, *mut *mut u8, c_int) -> i64>,
    pub encode_inplace_fn:
        Option<extern "C" fn(*mut cryptstate_T, *mut u8, usize, *mut u8, c_int)>,
    pub decode_inplace_fn:
        Option<extern "C" fn(*mut cryptstate_T, *mut u8, usize, *mut u8, c_int)>,
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
    use std::ffi::CString;

    #[test]
    fn blowfish_roundtrip() {
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
        let mut state_enc = cryptstate_T { method_nr: CRYPT_M_BF2, method_state: std::ptr::null_mut() };
        assert_eq!(1, crypt_blowfish_init(&mut state_enc, key.as_ptr(), &mut arg));
        let data = b"hello world";
        let mut enc = vec![0u8; data.len()];
        crypt_blowfish_encode(&mut state_enc, data.as_ptr(), data.len(), enc.as_mut_ptr(), 1);
        let mut state_dec = cryptstate_T { method_nr: CRYPT_M_BF2, method_state: std::ptr::null_mut() };
        assert_eq!(1, crypt_blowfish_init(&mut state_dec, key.as_ptr(), &mut arg));
        let mut dec = vec![0u8; data.len()];
        crypt_blowfish_decode(&mut state_dec, enc.as_ptr(), enc.len(), dec.as_mut_ptr(), 1);
        assert_eq!(data.to_vec(), dec);
    }
}
