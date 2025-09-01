#include "vim.h"

#if defined(FEAT_EVAL) || defined(PROTO)

typedef struct {
    char_u *result;
    char_u *error;
} json_result_T;

// Declarations for Rust FFI functions.
extern json_result_T json_encode_rs(const char *input);
extern json_result_T json_decode_rs(const char *input);
extern int json_find_end_rs(const char *input);

static char_u *
handle_encode_result(json_result_T r)
{
    if (r.error != NULL)
    {
        emsg((char *)r.error);
        vim_free(r.error);
        return NULL;
    }
    return r.result;
}

static int
handle_decode_result(json_result_T r, typval_T *res)
{
    if (r.error != NULL)
    {
        emsg((char *)r.error);
        vim_free(r.error);
        res->v_type = VAR_UNKNOWN;
        return FAIL;
    }
    res->v_type = VAR_STRING;
    res->vval.v_string = r.result;
    return OK;
}

// Encode a typval to JSON using the Rust implementation.
char_u *
json_encode(typval_T *val, int options)
{
    (void)options;
    return handle_encode_result(json_encode_rs((const char *)tv_get_string(val)));
}

// Encode [nr, val] expression; for now encode the value only.
char_u *
json_encode_nr_expr(int nr, typval_T *val, int options)
{
    (void)nr;
    return json_encode(val, options);
}

// Encode an LSP message; encode the value directly.
char_u *
json_encode_lsp_msg(typval_T *val)
{
    return json_encode(val, 0);
}

// Decode JSON using the Rust implementation.  The result is stored in "res".
int
json_decode(js_read_T *reader, typval_T *res, int options)
{
    (void)options;
    return handle_decode_result(json_decode_rs((const char *)reader->js_buf), res);
}

// Find the end of a JSON message using Rust implementation.
int
json_find_end(js_read_T *reader, int options)
{
    (void)options;
    return json_find_end_rs((const char *)reader->js_buf);
}

// Vim script interfaces -----------------------------------------------------

void
f_js_decode(typval_T *argvars, typval_T *rettv)
{
    (void)handle_decode_result(
            json_decode_rs((const char *)tv_get_string(&argvars[0])), rettv);
}

void
f_js_encode(typval_T *argvars, typval_T *rettv)
{
    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = handle_encode_result(
            json_encode_rs((const char *)tv_get_string(&argvars[0])));
}

void
f_json_decode(typval_T *argvars, typval_T *rettv)
{
    (void)handle_decode_result(
            json_decode_rs((const char *)tv_get_string(&argvars[0])), rettv);
}

void
f_json_encode(typval_T *argvars, typval_T *rettv)
{
    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = json_encode(&argvars[0], 0);
}

#endif
