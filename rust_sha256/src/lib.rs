use std::os::raw::c_uint;
use std::slice;

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

fn rotr(x: u32, n: u32) -> u32 { (x >> n) | (x << (32 - n)) }

fn sha256_process(ctx: &mut context_sha256_T, data: &[u8; 64]) {
    let mut w = [0u32; 64];
    for i in 0..16 {
        let j = i * 4;
        w[i] = u32::from_be_bytes([data[j], data[j + 1], data[j + 2], data[j + 3]]);
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
    if input.is_empty() { return; }
    let mut left = (ctx.total[0] & 0x3f) as usize;
    let fill = 64 - left;
    ctx.total[0] = ctx.total[0].wrapping_add(input.len() as u32);
    if ctx.total[0] < input.len() as u32 { ctx.total[1] = ctx.total[1].wrapping_add(1); }
    let mut data = input;
    if left != 0 && data.len() >= fill {
        ctx.buffer[left..left+fill].copy_from_slice(&data[..fill]);
        sha256_process(ctx, unsafe { &*(ctx.buffer.as_ptr() as *const [u8;64]) });
        data = &data[fill..];
        left = 0;
    }
    while data.len() >= 64 {
        let mut block = [0u8;64];
        block.copy_from_slice(&data[..64]);
        sha256_process(ctx, &block);
        data = &data[64..];
    }
    if !data.is_empty() {
        ctx.buffer[left..left+data.len()].copy_from_slice(data);
    }
}

fn sha256_finish_rust(ctx: &mut context_sha256_T, digest: &mut [u8;32]) {
    let mut msglen = [0u8;8];
    let high = (ctx.total[0] >> 29) | (ctx.total[1] << 3);
    let low = ctx.total[0] << 3;
    msglen[..4].copy_from_slice(&high.to_be_bytes());
    msglen[4..].copy_from_slice(&low.to_be_bytes());
    let last = (ctx.total[0] & 0x3f) as usize;
    let padn = if last < 56 {56 - last} else {120 - last};
    sha256_update_rust(ctx, &SHA256_PADDING[..padn]);
    sha256_update_rust(ctx, &msglen);
    for (i, chunk) in digest.chunks_mut(4).enumerate() {
        chunk.copy_from_slice(&ctx.state[i].to_be_bytes());
    }
}

static SHA256_PADDING: [u8;64] = [
    0x80, 0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
];

#[no_mangle]
pub extern "C" fn sha256_start(ctx: *mut context_sha256_T) {
    if ctx.is_null() { return; }
    sha256_start_rust(unsafe { &mut *ctx });
}

#[no_mangle]
pub extern "C" fn sha256_update(ctx: *mut context_sha256_T, input: *mut u8, length: c_uint) {
    if ctx.is_null() || input.is_null() || length == 0 { return; }
    let data = unsafe { slice::from_raw_parts(input, length as usize) };
    sha256_update_rust(unsafe { &mut *ctx }, data);
}

#[no_mangle]
pub extern "C" fn sha256_finish(ctx: *mut context_sha256_T, digest: *mut u8) {
    if ctx.is_null() || digest.is_null() { return; }
    let mut out = [0u8;32];
    sha256_finish_rust(unsafe { &mut *ctx }, &mut out);
    unsafe { slice::from_raw_parts_mut(digest, 32).copy_from_slice(&out); }
}

/// Compute SHA-256 digest of arbitrary data and return 32-byte array.
pub fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let mut ctx = context_sha256_T { total: [0;2], state: [0;8], buffer: [0;64] };
    sha256_start_rust(&mut ctx);
    sha256_update_rust(&mut ctx, data);
    let mut out = [0u8;32];
    sha256_finish_rust(&mut ctx, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_abc() {
        let mut ctx = context_sha256_T { total:[0;2], state:[0;8], buffer:[0;64] };
        unsafe {
            sha256_start(&mut ctx);
            let mut data = b"abc".to_vec();
            sha256_update(&mut ctx, data.as_mut_ptr(), data.len() as c_uint);
            let mut out = [0u8;32];
            sha256_finish(&mut ctx, out.as_mut_ptr());
            let hex: String = out.iter().map(|b| format!("{:02x}", b)).collect();
            assert_eq!(hex, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        }
    }
}
