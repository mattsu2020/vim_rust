use std::io::{Cursor, Read, Write};
use std::slice;

use aes::Aes128;
use cbc::{Decryptor, Encryptor};
use crypto::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use zip::{read::ZipArchive, write::FileOptions, CompressionMethod, ZipWriter};

// Internal helper for AES-128-CBC encryption.
fn aes_encrypt(data: &[u8], key: &[u8]) -> Option<Vec<u8>> {
    if key.len() != 16 {
        return None;
    }
    let iv = [0u8; 16];
    let mut out = vec![0u8; data.len() + 16];
    let cipher = Encryptor::<Aes128>::new(key.into(), &iv.into());
    let n = cipher
        .encrypt_padded_b2b_mut::<Pkcs7>(data, &mut out)
        .ok()?
        .len();
    out.truncate(n);
    Some(out)
}

// Internal helper for AES-128-CBC decryption.
fn aes_decrypt(data: &[u8], key: &[u8]) -> Option<Vec<u8>> {
    if key.len() != 16 {
        return None;
    }
    let iv = [0u8; 16];
    let mut out = vec![0u8; data.len()];
    let cipher = Decryptor::<Aes128>::new(key.into(), &iv.into());
    let n = cipher
        .decrypt_padded_b2b_mut::<Pkcs7>(data, &mut out)
        .ok()?
        .len();
    out.truncate(n);
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

// FFI: encrypt buffer using AES-128-CBC with PKCS#7 padding.
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
    if let Some(enc) = aes_encrypt(input, key) {
        if enc.len() > out_slice.len() {
            return 0;
        }
        out_slice[..enc.len()].copy_from_slice(&enc);
        enc.len()
    } else {
        0
    }
}

// FFI: decrypt buffer using AES-128-CBC with PKCS#7 padding.
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
    if let Some(dec) = aes_decrypt(input, key) {
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
}
