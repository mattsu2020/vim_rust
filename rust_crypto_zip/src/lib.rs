use std::io::{Cursor, Read, Write};
use std::slice;
use std::sync::Once;

use zip::{read::ZipArchive, write::FileOptions, CompressionMethod, ZipWriter};

// The classic Zip crypto algorithm uses a CRC table and three 32-bit keys
// that are updated for each processed byte.  The table only needs to be
// generated once.
static mut CRC_TABLE: [u32; 256] = [0; 256];
static INIT: Once = Once::new();

fn make_crc_tab() {
    INIT.call_once(|| unsafe {
        for t in 0..256 {
            let mut v = t as u32;
            for _ in 0..8 {
                v = (v >> 1) ^ ((v & 1) * 0xedb88320);
            }
            CRC_TABLE[t] = v;
        }
    });
}

fn crc32(c: u32, b: u8) -> u32 {
    unsafe { CRC_TABLE[(c as u8 ^ b) as usize] ^ (c >> 8) }
}

fn update_keys(keys: &mut [u32; 3], c: u8) {
    keys[0] = crc32(keys[0], c);
    keys[1] = keys[1].wrapping_add(keys[0] & 0xff);
    keys[1] = keys[1].wrapping_mul(134775813).wrapping_add(1);
    keys[2] = crc32(keys[2], (keys[1] >> 24) as u8);
}

fn decrypt_byte(keys: &[u32; 3]) -> u8 {
    let temp = (keys[2] | 2) as u16;
    (((temp as u32) * ((temp ^ 1) as u32) >> 8) & 0xff) as u8
}

// Encrypt data using the traditional Zip crypto algorithm.
fn zip_encrypt(data: &[u8], key: &[u8]) -> Option<Vec<u8>> {
    make_crc_tab();
    let mut keys = [305419896u32, 591751049u32, 878082192u32];
    for &b in key {
        update_keys(&mut keys, b);
    }
    let mut out = Vec::with_capacity(data.len());
    for &b in data {
        let t = decrypt_byte(&keys);
        update_keys(&mut keys, b);
        out.push(t ^ b);
    }
    Some(out)
}

// Decrypt data using the traditional Zip crypto algorithm.
fn zip_decrypt(data: &[u8], key: &[u8]) -> Option<Vec<u8>> {
    make_crc_tab();
    let mut keys = [305419896u32, 591751049u32, 878082192u32];
    for &b in key {
        update_keys(&mut keys, b);
    }
    let mut out = Vec::with_capacity(data.len());
    for &b in data {
        let t = decrypt_byte(&keys);
        let val = b ^ t;
        update_keys(&mut keys, val);
        out.push(val);
    }
    Some(out)
}

// Compress data into a single-file ZIP archive.
fn zip_compress(data: &[u8]) -> Option<Vec<u8>> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = ZipWriter::new(&mut cursor);
        let options = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);
        writer.start_file("data", options).ok()?;
        writer.write_all(data).ok()?;
        writer.finish().ok()?;
    }
    Some(cursor.into_inner())
}

// Decompress a single-file ZIP archive.
fn zip_decompress(data: &[u8]) -> Option<Vec<u8>> {
    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor).ok()?;
    let mut file = archive.by_index(0).ok()?;
    let mut out = Vec::new();
    file.read_to_end(&mut out).ok()?;
    Some(out)
}

// FFI: encrypt buffer using the classic Zip crypto algorithm.
#[no_mangle]
pub extern "C" fn rs_encrypt(
    input: *const u8,
    input_len: usize,
    key: *const u8,
    key_len: usize,
    output: *mut u8,
    output_len: usize,
) -> usize {
    if input.is_null() || key.is_null() || output.is_null() {
        return 0;
    }
    let input = unsafe { slice::from_raw_parts(input, input_len) };
    let key = unsafe { slice::from_raw_parts(key, key_len) };
    let out_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };
    if let Some(enc) = zip_encrypt(input, key) {
        if enc.len() > out_slice.len() {
            return 0;
        }
        out_slice[..enc.len()].copy_from_slice(&enc);
        enc.len()
    } else {
        0
    }
}

// FFI: decrypt buffer using the classic Zip crypto algorithm.
#[no_mangle]
pub extern "C" fn rs_decrypt(
    input: *const u8,
    input_len: usize,
    key: *const u8,
    key_len: usize,
    output: *mut u8,
    output_len: usize,
) -> usize {
    if input.is_null() || key.is_null() || output.is_null() {
        return 0;
    }
    let input = unsafe { slice::from_raw_parts(input, input_len) };
    let key = unsafe { slice::from_raw_parts(key, key_len) };
    let out_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };
    if let Some(dec) = zip_decrypt(input, key) {
        if dec.len() > out_slice.len() {
            return 0;
        }
        out_slice[..dec.len()].copy_from_slice(&dec);
        dec.len()
    } else {
        0
    }
}

// FFI: compress data into ZIP format.
#[no_mangle]
pub extern "C" fn rs_zip_compress(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
) -> usize {
    if input.is_null() || output.is_null() {
        return 0;
    }
    let input = unsafe { slice::from_raw_parts(input, input_len) };
    let out_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };
    if let Some(comp) = zip_compress(input) {
        if comp.len() > out_slice.len() {
            return 0;
        }
        out_slice[..comp.len()].copy_from_slice(&comp);
        comp.len()
    } else {
        0
    }
}

// FFI: decompress ZIP data.
#[no_mangle]
pub extern "C" fn rs_zip_decompress(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
) -> usize {
    if input.is_null() || output.is_null() {
        return 0;
    }
    let input = unsafe { slice::from_raw_parts(input, input_len) };
    let out_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };
    if let Some(decomp) = zip_decompress(input) {
        if decomp.len() > out_slice.len() {
            return 0;
        }
        out_slice[..decomp.len()].copy_from_slice(&decomp);
        decomp.len()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_roundtrip() {
        let data = b"hello rust";
        let key = [0x11u8; 16];
        let mut enc_buf = vec![0u8; data.len() + 16];
        let enc_len = rs_encrypt(
            data.as_ptr(),
            data.len(),
            key.as_ptr(),
            key.len(),
            enc_buf.as_mut_ptr(),
            enc_buf.len(),
        );
        assert!(enc_len > 0);
        let mut dec_buf = vec![0u8; enc_len];
        let dec_len = rs_decrypt(
            enc_buf.as_ptr(),
            enc_len,
            key.as_ptr(),
            key.len(),
            dec_buf.as_mut_ptr(),
            dec_buf.len(),
        );
        assert_eq!(dec_len, data.len());
        assert_eq!(&dec_buf[..dec_len], data);
    }

    #[test]
    fn zip_roundtrip() {
        let data = b"zip me";
        let mut comp = vec![0u8; 512];
        let clen = rs_zip_compress(data.as_ptr(), data.len(), comp.as_mut_ptr(), comp.len());
        assert!(clen > 0);
        let mut decomp = vec![0u8; 512];
        let dlen = rs_zip_decompress(comp.as_ptr(), clen, decomp.as_mut_ptr(), decomp.len());
        assert_eq!(dlen, data.len());
        assert_eq!(&decomp[..dlen], data);
    }

    #[test]
    fn compatibility_vector() {
        let key = b"foofoo";
        let plaintext = b"1234567890\na\xe1bbccdde\xebff\n";
        let ciphertext: [u8; 24] = [
            6, 28, 108, 86, 39, 222, 125, 77, 103, 160, 234, 163, 86, 169, 231,
            7, 69, 35, 51, 142, 50, 85, 233, 151,
        ];

        let mut enc_buf = vec![0u8; plaintext.len()];
        let enc_len = rs_encrypt(
            plaintext.as_ptr(),
            plaintext.len(),
            key.as_ptr(),
            key.len(),
            enc_buf.as_mut_ptr(),
            enc_buf.len(),
        );
        assert_eq!(enc_len, ciphertext.len());
        assert_eq!(&enc_buf[..enc_len], &ciphertext);

        let mut dec_buf = vec![0u8; ciphertext.len()];
        let dec_len = rs_decrypt(
            ciphertext.as_ptr(),
            ciphertext.len(),
            key.as_ptr(),
            key.len(),
            dec_buf.as_mut_ptr(),
            dec_buf.len(),
        );
        assert_eq!(dec_len, plaintext.len());
        assert_eq!(&dec_buf[..dec_len], plaintext);
    }
}
