#ifndef CRYPT_RS_H
#define CRYPT_RS_H

// structs.h is included via vim.h in C sources; avoid double-including here.
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

struct cryptmethod_S;
typedef struct cryptmethod_S cryptmethod_T;
cryptmethod_T *rust_crypt_methods(void);
void crypt_state_free(cryptstate_T *state);

// AES-256-GCM convenience wrappers implemented in Rust.
size_t rust_crypt_encrypt(const uint8_t *input, size_t input_len,
                          const uint8_t *key, size_t key_len,
                          uint8_t *output, size_t output_len);
size_t rust_crypt_decrypt(const uint8_t *input, size_t input_len,
                          const uint8_t *key, size_t key_len,
                          uint8_t *output, size_t output_len);

#ifdef __cplusplus
}
#endif

#endif // CRYPT_RS_H
