use ring::aead::{self, Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use std::slice;

#[no_mangle]
pub extern "C" fn rust_crypt_encrypt(
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
    let data = unsafe { slice::from_raw_parts(input, input_len) };
    let key_slice = unsafe { slice::from_raw_parts(key, key_len) };
    if key_slice.len() != 32 {
        return 0;
    }
    let unbound = match UnboundKey::new(&AES_256_GCM, key_slice) {
        Ok(k) => k,
        Err(_) => return 0,
    };
    let key = LessSafeKey::new(unbound);
    let nonce_bytes = [0u8; 12];
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let mut in_out = data.to_vec();
    match key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out) {
        Ok(_) => {
            if in_out.len() > output_len {
                return 0;
            }
            unsafe { std::ptr::copy_nonoverlapping(in_out.as_ptr(), output, in_out.len()); }
            in_out.len()
        }
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn rust_crypt_decrypt(
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
    let mut in_out = unsafe { slice::from_raw_parts(input, input_len).to_vec() };
    let key_slice = unsafe { slice::from_raw_parts(key, key_len) };
    if key_slice.len() != 32 {
        return 0;
    }
    let unbound = match UnboundKey::new(&AES_256_GCM, key_slice) {
        Ok(k) => k,
        Err(_) => return 0,
    };
    let key = LessSafeKey::new(unbound);
    let nonce_bytes = [0u8; 12];
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let plaintext = match key.open_in_place(nonce, Aad::empty(), &mut in_out) {
        Ok(p) => p,
        Err(_) => return 0,
    };
    if plaintext.len() > output_len {
        return 0;
    }
    unsafe { std::ptr::copy_nonoverlapping(plaintext.as_ptr(), output, plaintext.len()); }
    plaintext.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let key = [0u8; 32];
        let msg = b"hello rust";
        let mut enc = vec![0u8; msg.len() + 16];
        let enc_len = rust_crypt_encrypt(
            msg.as_ptr(), msg.len(), key.as_ptr(), key.len(), enc.as_mut_ptr(), enc.len());
        assert!(enc_len > 0);
        enc.truncate(enc_len);
        let mut dec = vec![0u8; msg.len()];
        let dec_len = rust_crypt_decrypt(
            enc.as_ptr(), enc.len(), key.as_ptr(), key.len(), dec.as_mut_ptr(), dec.len());
        assert_eq!(dec_len, msg.len());
        dec.truncate(dec_len);
        assert_eq!(msg.to_vec(), dec);
    }
}

