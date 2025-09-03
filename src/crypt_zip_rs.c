#include "rust_crypto_zip.h"

// C wrappers around the Rust FFI functions to maintain
// compatibility with existing C code during migration.

size_t crypt_rust_encrypt(const uint8_t *data, size_t len,
                          const uint8_t *key, size_t key_len,
                          uint8_t *out, size_t out_len)
{
    return rs_encrypt(data, len, key, key_len, out, out_len);
}

size_t crypt_rust_decrypt(const uint8_t *data, size_t len,
                          const uint8_t *key, size_t key_len,
                          uint8_t *out, size_t out_len)
{
    return rs_decrypt(data, len, key, key_len, out, out_len);
}

size_t zip_rust_compress(const uint8_t *data, size_t len,
                         uint8_t *out, size_t out_len)
{
    return rs_zip_compress(data, len, out, out_len);
}

size_t zip_rust_decompress(const uint8_t *data, size_t len,
                           uint8_t *out, size_t out_len)
{
    return rs_zip_decompress(data, len, out, out_len);
}
