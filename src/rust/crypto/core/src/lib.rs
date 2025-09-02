use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::slice;
use std::sync::Once;

use blowfish::cipher::{generic_array::GenericArray, BlockEncrypt, NewBlockCipher};
use blowfish::Blowfish;

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

const CRYPT_M_ZIP: c_int = 0;
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
    let mut ctx = context_sha256_T { total: [0; 2], state: [0; 8], buffer: [0; 64] };
    sha256_start_rust(&mut ctx);
    sha256_update_rust(&mut ctx, data);
    if !salt.is_empty() {
        sha256_update_rust(&mut ctx, salt);
    }
    let mut out = [0u8; 32];
    sha256_finish_rust(&mut ctx, &mut out);
    let mut s = String::with_capacity(64);
    for b in &out {
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

struct ZipState {
    keys: [u32; 3],
}

static mut CRC_TABLE: [u32; 256] = [0; 256];
static CRC_INIT: Once = Once::new();

fn make_crc_tab() {
    unsafe {
        CRC_INIT.call_once(|| {
            for t in 0..256u32 {
                let mut v = t;
                for _ in 0..8 {
                    v = (v >> 1) ^ ((v & 1) * 0xedb88320u32);
                }
                CRC_TABLE[t as usize] = v;
            }
        });
    }
}

fn crc32(c: u32, b: u8) -> u32 {
    unsafe { CRC_TABLE[((c as u8) ^ b) as usize] ^ (c >> 8) }
}

fn decrypt_byte_zip(keys: &[u32; 3]) -> u8 {
    let temp = (keys[2] as u16 | 2) as u32;
    (((temp.wrapping_mul(temp ^ 1)) >> 8) & 0xff) as u8
}

fn update_keys_zip(keys: &mut [u32; 3], c: u8) {
    keys[0] = crc32(keys[0], c);
    keys[1] = keys[1].wrapping_add(keys[0] & 0xff);
    keys[1] = keys[1].wrapping_mul(134775813).wrapping_add(1);
    keys[2] = crc32(keys[2], (keys[1] >> 24) as u8);
}

#[no_mangle]
pub extern "C" fn crypt_zip_init(
    state: *mut cryptstate_T,
    key: *const c_char,
    _arg: *mut crypt_arg_T,
) -> c_int {
    if state.is_null() || key.is_null() {
        return 0;
    }
    make_crc_tab();
    let key_bytes = unsafe { CStr::from_ptr(key).to_bytes() };
    let mut st = ZipState {
        keys: [305419896, 591751049, 878082192],
    };
    for &b in key_bytes {
        update_keys_zip(&mut st.keys, b);
    }
    unsafe {
        (*state).method_state = Box::into_raw(Box::new(st)) as *mut c_void;
    }
    1
}

#[no_mangle]
pub extern "C" fn crypt_zip_encode(
    state: *mut cryptstate_T,
    from: *const u8,
    len: usize,
    to: *mut u8,
    _last: c_int,
) {
    if state.is_null() || from.is_null() || to.is_null() {
        return;
    }
    let st = unsafe { &mut *(*state).method_state.cast::<ZipState>() };
    let from_slice = unsafe { slice::from_raw_parts(from, len) };
    let to_slice = unsafe { slice::from_raw_parts_mut(to, len) };
    for i in 0..len {
        let ztemp = from_slice[i];
        let t = decrypt_byte_zip(&st.keys);
        update_keys_zip(&mut st.keys, ztemp);
        to_slice[i] = t ^ ztemp;
    }
}

#[no_mangle]
pub extern "C" fn crypt_zip_decode(
    state: *mut cryptstate_T,
    from: *const u8,
    len: usize,
    to: *mut u8,
    _last: c_int,
) {
    if state.is_null() || from.is_null() || to.is_null() {
        return;
    }
    let st = unsafe { &mut *(*state).method_state.cast::<ZipState>() };
    let from_slice = unsafe { slice::from_raw_parts(from, len) };
    let to_slice = unsafe { slice::from_raw_parts_mut(to, len) };
    for i in 0..len {
        let t = decrypt_byte_zip(&st.keys);
        let val = from_slice[i] ^ t;
        update_keys_zip(&mut st.keys, val);
        to_slice[i] = val;
    }
}

#[no_mangle]
pub extern "C" fn crypt_zip_encode_inplace(
    state: *mut cryptstate_T,
    buf: *mut u8,
    len: usize,
    _p2: *mut u8,
    last: c_int,
) {
    crypt_zip_encode(state, buf as *const u8, len, buf, last);
}

#[no_mangle]
pub extern "C" fn crypt_zip_decode_inplace(
    state: *mut cryptstate_T,
    buf: *mut u8,
    len: usize,
    _p2: *mut u8,
    last: c_int,
) {
    crypt_zip_decode(state, buf as *const u8, len, buf, last);
}

#[repr(C)]
pub struct context_sha256_T {
    total: [u32; 2],
    state: [u32; 8],
    buffer: [u8; 64],
}

const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1,
    0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
    0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
    0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
    0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
    0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_start_rust(ctx: &mut context_sha256_T) {
    ctx.total = [0, 0];
    ctx.state = [
        0x6a09e667,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
}

fn sha256_process(ctx: &mut context_sha256_T, data: &[u8; 64]) {
    fn rotr(x: u32, n: u32) -> u32 {
        (x >> n) | (x << (32 - n))
    }

    let mut w = [0u32; 64];
    for i in 0..16 {
        let j = i * 4;
        w[i] = u32::from_be_bytes([
            data[j],
            data[j + 1],
            data[j + 2],
            data[j + 3],
        ]);
    }
    for t in 16..64 {
        let s0 = rotr(w[t - 15], 7) ^ rotr(w[t - 15], 18) ^ (w[t - 15] >> 3);
        let s1 = rotr(w[t - 2], 17) ^ rotr(w[t - 2], 19) ^ (w[t - 2] >> 10);
        w[t] = w[t - 16]
            .wrapping_add(s0)
            .wrapping_add(w[t - 7])
            .wrapping_add(s1);
    }

    let mut a = ctx.state[0];
    let mut b = ctx.state[1];
    let mut c = ctx.state[2];
    let mut d = ctx.state[3];
    let mut e = ctx.state[4];
    let mut f = ctx.state[5];
    let mut g = ctx.state[6];
    let mut h = ctx.state[7];

    for i in 0..64 {
        let s1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
        let ch = (e & f) ^ ((!e) & g);
        let temp1 = h
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(K256[i])
            .wrapping_add(w[i]);
        let s0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(maj);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    ctx.state[0] = ctx.state[0].wrapping_add(a);
    ctx.state[1] = ctx.state[1].wrapping_add(b);
    ctx.state[2] = ctx.state[2].wrapping_add(c);
    ctx.state[3] = ctx.state[3].wrapping_add(d);
    ctx.state[4] = ctx.state[4].wrapping_add(e);
    ctx.state[5] = ctx.state[5].wrapping_add(f);
    ctx.state[6] = ctx.state[6].wrapping_add(g);
    ctx.state[7] = ctx.state[7].wrapping_add(h);
}

fn sha256_update_rust(ctx: &mut context_sha256_T, input: &[u8]) {
    if input.is_empty() {
        return;
    }
    let mut left = (ctx.total[0] & 0x3f) as usize;
    let fill = 64 - left;

    ctx.total[0] = ctx.total[0].wrapping_add(input.len() as u32);
    if ctx.total[0] < input.len() as u32 {
        ctx.total[1] = ctx.total[1].wrapping_add(1);
    }

    let mut data = input;
    if left != 0 && data.len() >= fill {
        ctx.buffer[left..left + fill].copy_from_slice(&data[..fill]);
        sha256_process(ctx, unsafe { &*(ctx.buffer.as_ptr() as *const [u8; 64]) });
        data = &data[fill..];
        left = 0;
    }
    while data.len() >= 64 {
        let mut block = [0u8; 64];
        block.copy_from_slice(&data[..64]);
        sha256_process(ctx, &block);
        data = &data[64..];
    }
    if !data.is_empty() {
        ctx.buffer[left..left + data.len()].copy_from_slice(data);
    }
}

fn sha256_finish_rust(ctx: &mut context_sha256_T, digest: &mut [u8; 32]) {
    let mut msglen = [0u8; 8];
    let high = (ctx.total[0] >> 29) | (ctx.total[1] << 3);
    let low = ctx.total[0] << 3;
    msglen[..4].copy_from_slice(&high.to_be_bytes());
    msglen[4..].copy_from_slice(&low.to_be_bytes());

    let last = (ctx.total[0] & 0x3f) as usize;
    let padn = if last < 56 { 56 - last } else { 120 - last };
    sha256_update_rust(ctx, &SHA256_PADDING[..padn]);
    sha256_update_rust(ctx, &msglen);

    for (i, chunk) in digest.chunks_mut(4).enumerate() {
        chunk.copy_from_slice(&ctx.state[i].to_be_bytes());
    }
}

static SHA256_PADDING: [u8; 64] = [
    0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

#[no_mangle]
pub extern "C" fn sha256_start(ctx: *mut context_sha256_T) {
    if ctx.is_null() {
        return;
    }
    sha256_start_rust(unsafe { &mut *ctx });
}

#[no_mangle]
pub extern "C" fn sha256_update(
    ctx: *mut context_sha256_T,
    input: *mut u8,
    length: u32,
) {
    if ctx.is_null() || input.is_null() || length == 0 {
        return;
    }
    let data = unsafe { slice::from_raw_parts(input, length as usize) };
    sha256_update_rust(unsafe { &mut *ctx }, data);
}

#[no_mangle]
pub extern "C" fn sha256_finish(ctx: *mut context_sha256_T, digest: *mut u8) {
    if ctx.is_null() || digest.is_null() {
        return;
    }
    let mut out = [0u8; 32];
    sha256_finish_rust(unsafe { &mut *ctx }, &mut out);
    unsafe {
        slice::from_raw_parts_mut(digest, 32).copy_from_slice(&out);
    }
}

static mut SHA256_HEX: [u8; 65] = [0; 65];

#[no_mangle]
pub extern "C" fn sha256_bytes(
    buf: *mut u8,
    buf_len: c_int,
    salt: *mut u8,
    salt_len: c_int,
) -> *mut u8 {
    unsafe {
        let data = slice::from_raw_parts(buf, buf_len as usize);
        let salt_slice = if salt.is_null() {
            &[]
        } else {
            slice::from_raw_parts(salt, salt_len as usize)
        };
        let mut ctx = context_sha256_T {
            total: [0; 2],
            state: [0; 8],
            buffer: [0; 64],
        };
        sha256_start_rust(&mut ctx);
        sha256_update_rust(&mut ctx, data);
        if !salt_slice.is_empty() {
            sha256_update_rust(&mut ctx, salt_slice);
        }
        let mut out = [0u8; 32];
        sha256_finish_rust(&mut ctx, &mut out);
        for (i, b) in out.iter().enumerate() {
            SHA256_HEX[i * 2] = b"0123456789abcdef"[(b >> 4) as usize];
            SHA256_HEX[i * 2 + 1] = b"0123456789abcdef"[(b & 0xf) as usize];
        }
        SHA256_HEX[64] = 0;
        SHA256_HEX.as_mut_ptr()
    }
}

#[no_mangle]
pub extern "C" fn sha256_key(
    buf: *mut u8,
    salt: *mut u8,
    salt_len: c_int,
) -> *mut u8 {
    if buf.is_null() || unsafe { *buf } == 0 {
        return b"\0".as_ptr() as *mut u8;
    }
    unsafe {
        let len = libc::strlen(buf as *const i8);
        sha256_bytes(buf, len as c_int, salt, salt_len)
    }
}

static mut SHA256_SELF_TESTED: c_int = 0;
static mut SHA256_FAILURES: c_int = 0;

#[no_mangle]
pub extern "C" fn sha256_self_test() -> c_int {
    unsafe {
        if SHA256_SELF_TESTED > 0 {
            return if SHA256_FAILURES > 0 { 0 } else { 1 };
        }
        SHA256_SELF_TESTED = 1;
        const MSGS: [&[u8]; 2] = [b"abc", b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"];
        const VECTORS: [&str; 2] = [
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ];
        for (msg, vec) in MSGS.iter().zip(VECTORS.iter()) {
            let mut ctx = context_sha256_T { total: [0; 2], state: [0; 8], buffer: [0; 64] };
            sha256_start_rust(&mut ctx);
            sha256_update_rust(&mut ctx, msg);
            let mut out = [0u8; 32];
            sha256_finish_rust(&mut ctx, &mut out);
            let mut hex = String::with_capacity(64);
            for b in &out {
                hex.push_str(&format!("{:02x}", b));
            }
            if hex != *vec {
                SHA256_FAILURES += 1;
            }
        }
        if SHA256_FAILURES > 0 { 0 } else { 1 }
    }
}

fn get_some_time() -> u32 {
    unsafe {
        #[cfg(any(unix, target_os = "wasi"))]
        {
            let mut tv: libc::timeval = std::mem::zeroed();
            if libc::gettimeofday(&mut tv, std::ptr::null_mut()) == 0 {
                return (tv.tv_sec as u32).wrapping_add(tv.tv_usec as u32);
            }
        }
        libc::time(std::ptr::null_mut()) as u32
    }
}

#[no_mangle]
pub extern "C" fn sha2_seed(
    header: *mut u8,
    header_len: c_int,
    salt: *mut u8,
    salt_len: c_int,
) {
    if header.is_null() {
        return;
    }
    unsafe {
        libc::srand(get_some_time());
        let mut random_data = [0u8; 1000];
        for i in 0..random_data.len() - 1 {
            random_data[i] = ((get_some_time() ^ libc::rand() as u32) & 0xff) as u8;
        }
        let mut ctx = context_sha256_T { total: [0; 2], state: [0; 8], buffer: [0; 64] };
        sha256_start_rust(&mut ctx);
        sha256_update_rust(&mut ctx, &random_data);
        let mut sha256sum = [0u8; 32];
        sha256_finish_rust(&mut ctx, &mut sha256sum);
        for i in 0..header_len as usize {
            *header.add(i) = sha256sum[i % 32];
        }
        if !salt.is_null() {
            for i in 0..salt_len as usize {
                *salt.add(i) = sha256sum[(i + header_len as usize) % 32];
            }
        }
    }
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
        init_fn: Some(crypt_zip_init),
        encode_fn: Some(crypt_zip_encode),
        decode_fn: Some(crypt_zip_decode),
        encode_buffer_fn: None,
        decode_buffer_fn: None,
        encode_inplace_fn: Some(crypt_zip_encode_inplace),
        decode_inplace_fn: Some(crypt_zip_decode_inplace),
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

    #[test]
    fn zip_roundtrip() {
        let key = CString::new("test").unwrap();
        let mut state_enc = cryptstate_T { method_nr: CRYPT_M_ZIP, method_state: std::ptr::null_mut() };
        assert_eq!(1, crypt_zip_init(&mut state_enc, key.as_ptr(), std::ptr::null_mut()));
        let data = b"hello world";
        let mut enc = vec![0u8; data.len()];
        crypt_zip_encode(&mut state_enc, data.as_ptr(), data.len(), enc.as_mut_ptr(), 1);
        let mut state_dec = cryptstate_T { method_nr: CRYPT_M_ZIP, method_state: std::ptr::null_mut() };
        assert_eq!(1, crypt_zip_init(&mut state_dec, key.as_ptr(), std::ptr::null_mut()));
        let mut dec = vec![0u8; data.len()];
        crypt_zip_decode(&mut state_dec, enc.as_ptr(), enc.len(), dec.as_mut_ptr(), 1);
        assert_eq!(data.to_vec(), dec);
    }
}
