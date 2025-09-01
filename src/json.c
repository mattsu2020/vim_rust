#include "vim.h"

#if defined(FEAT_EVAL) || defined(PROTO)

// Structures and declarations for Rust FFI functions.
typedef struct {
    char_u *ptr;
    size_t used;
    int error;
} JsonDecodeResult;

typedef struct {
    size_t used;
    int status;
} JsonFindEndResult;

extern char_u *json_encode_rs(const char *input, int options);
extern JsonDecodeResult json_decode_rs(const char *input, int options);
extern JsonFindEndResult json_find_end_rs(const char *input, int options);

// Basic JSON encode wrapper using Rust implementation.
char_u *
json_encode(typval_T *val, int options)
{
    return json_encode_rs((const char *)tv_get_string(val), options);
}

// Encode [nr, val] expression; simplified to encode the value only.
char_u *
json_encode_nr_expr(int nr, typval_T *val, int options)
{
    (void)nr;
    return json_encode(val, options);
}

// Encode an LSP message; simplified to encode the value directly.
char_u *
json_encode_lsp_msg(typval_T *val)
{
    return json_encode(val, 0);
}

// Decode JSON using the Rust implementation.  The result is stored as a
// string in "res".
int
json_decode(js_read_T *reader, typval_T *res, int options)
{
    JsonDecodeResult r = json_decode_rs((const char *)reader->js_buf, options);
    reader->js_used = (int)r.used;
    res->v_type = VAR_STRING;
    res->vval.v_string = r.ptr;
    if (r.error == 0)
        return OK;
    return r.error == 1 ? MAYBE : FAIL;
}

// Find the end of a JSON message using the Rust implementation.
int
json_find_end(js_read_T *reader, int options)
{
    JsonFindEndResult r = json_find_end_rs((const char *)reader->js_buf, options);
    reader->js_used = (int)r.used;
    if (r.status == 1)
        return OK;
    return r.status == 2 ? MAYBE : FAIL;
}

// Vim script interfaces -----------------------------------------------------

void
f_js_decode(typval_T *argvars, typval_T *rettv)
{
    JsonDecodeResult r = json_decode_rs((const char *)tv_get_string(&argvars[0]), 0);
    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = r.ptr;
}

void
f_js_encode(typval_T *argvars, typval_T *rettv)
{
    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = json_encode_rs((const char *)tv_get_string(&argvars[0]), 0);
}

void
f_json_decode(typval_T *argvars, typval_T *rettv)
{
    JsonDecodeResult r = json_decode_rs((const char *)tv_get_string(&argvars[0]), 0);
    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = r.ptr;
}

void
f_json_encode(typval_T *argvars, typval_T *rettv)
{
    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = json_encode(&argvars[0], 0);
}

#endif
