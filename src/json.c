#include "vim.h"

// FFI bindings to the Rust implementation in rust_json crate.
char *rs_json_encode(typval_T *val, int options);
char *rs_json_encode_nr_expr(long long nr);
int rs_json_decode(const char *s, typval_T *res);

// Encode a typval_T into a JSON string.
char_u *json_encode(typval_T *val, int options)
{
    return (char_u *)rs_json_encode(val, options);
}

// Encode a number expression as JSON. The "val" and "options" arguments
// are currently ignored by the Rust implementation.
char_u *json_encode_nr_expr(int nr, typval_T *val, int options)
{
    (void)val;
    (void)options;
    return (char_u *)rs_json_encode_nr_expr(nr);
}

// Convenience wrapper used by channel code.
char_u *json_encode_lsp_msg(typval_T *val)
{
    return json_encode(val, 0);
}

// Decode a JSON string from "reader" into "res".
int json_decode(js_read_T *reader, typval_T *res, int options)
{
    (void)options;
    return rs_json_decode((const char *)reader->js_buf, res);
}
