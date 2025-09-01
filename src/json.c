#include "vim.h"

#if defined(FEAT_EVAL) || defined(PROTO)

// Declarations for Rust FFI functions.
extern char_u *json_encode_rs(const char *input);
extern char_u *json_decode_rs(const char *input);

// Basic JSON encode wrapper using Rust implementation.
char_u *
json_encode(typval_T *val, int options)
{
    // Ignoring options for the simplified Rust implementation.
    return json_encode_rs((const char *)tv_get_string(val));
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
    (void)options;
    res->v_type = VAR_STRING;
    res->vval.v_string = json_decode_rs((const char *)reader->js_buf);
    return OK;
}

// Find the end of a JSON message; this stub always succeeds.
int
json_find_end(js_read_T *reader, int options)
{
    (void)reader;
    (void)options;
    return OK;
}

// Vim script interfaces -----------------------------------------------------

void
f_js_decode(typval_T *argvars, typval_T *rettv)
{
    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = json_decode_rs((const char *)tv_get_string(&argvars[0]));
}

void
f_js_encode(typval_T *argvars, typval_T *rettv)
{
    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = json_encode_rs((const char *)tv_get_string(&argvars[0]));
}

void
f_json_decode(typval_T *argvars, typval_T *rettv)
{
    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = json_decode_rs((const char *)tv_get_string(&argvars[0]));
}

void
f_json_encode(typval_T *argvars, typval_T *rettv)
{
    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = json_encode(&argvars[0], 0);
}

#endif
