#ifndef CRYPT_RS_H
#define CRYPT_RS_H

#include "structs.h"

#ifdef __cplusplus
extern "C" {
#endif

cryptmethod_T *rust_crypt_methods(void);
int crypt_zip_init(cryptstate_T *state, char_u *key, crypt_arg_T *arg);
void crypt_zip_encode(cryptstate_T *state, char_u *from, size_t len, char_u *to, int last);
void crypt_zip_decode(cryptstate_T *state, char_u *from, size_t len, char_u *to, int last);
int crypt_blowfish_init(cryptstate_T *state, char_u *key, crypt_arg_T *arg);
void crypt_blowfish_encode(cryptstate_T *state, char_u *from, size_t len, char_u *to, int last);
void crypt_blowfish_decode(cryptstate_T *state, char_u *from, size_t len, char_u *to, int last);
void crypt_blowfish_encode_inplace(cryptstate_T *state, char_u *buf, size_t len, char_u *p2, int last);
void crypt_blowfish_decode_inplace(cryptstate_T *state, char_u *buf, size_t len, char_u *p2, int last);

#ifdef __cplusplus
}
#endif

#endif // CRYPT_RS_H
