#ifndef RUST_CRYPTO_ZIP_H
#define RUST_CRYPTO_ZIP_H

#include <stddef.h>
#include <stdint.h>

// Encrypt input buffer into output using the traditional Zip crypto algorithm.
// Returns number of bytes written to output or 0 on error.
size_t rs_encrypt(const uint8_t *input, size_t input_len,
                  const uint8_t *key, size_t key_len,
                  uint8_t *output, size_t output_len);

// Decrypt input buffer into output using the traditional Zip crypto algorithm.
// Returns number of bytes written to output or 0 on error.
size_t rs_decrypt(const uint8_t *input, size_t input_len,
                  const uint8_t *key, size_t key_len,
                  uint8_t *output, size_t output_len);

// Compress input buffer into output as a single-file ZIP archive.
// Returns number of bytes written to output or 0 on error.
size_t rs_zip_compress(const uint8_t *input, size_t input_len,
                       uint8_t *output, size_t output_len);

// Decompress a single-file ZIP archive from input into output.
// Returns number of bytes written to output or 0 on error.
size_t rs_zip_decompress(const uint8_t *input, size_t input_len,
                         uint8_t *output, size_t output_len);

#endif // RUST_CRYPTO_ZIP_H
