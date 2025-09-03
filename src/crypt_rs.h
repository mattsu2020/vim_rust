#ifndef CRYPT_RS_H
#define CRYPT_RS_H

// structs.h is included via vim.h in C sources; avoid double-including here.

#ifdef __cplusplus
extern "C" {
#endif

struct cryptmethod_S;
typedef struct cryptmethod_S cryptmethod_T;
cryptmethod_T *rust_crypt_methods(void);
void crypt_state_free(cryptstate_T *state);

#ifdef __cplusplus
}
#endif

#endif // CRYPT_RS_H
