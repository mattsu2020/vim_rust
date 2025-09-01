/* json_test.c: Tests for Rust JSON implementation */
#include "vim.h"
#include <assert.h>
#include <string.h>

typedef struct {
    char_u *result;
    char_u *error;
} json_result_T;

extern json_result_T json_encode_rs(const char *input);
extern json_result_T json_decode_rs(const char *input);

int main(void)
{
    json_result_T enc = json_encode_rs("vim");
    assert(enc.error == NULL);
    assert(strcmp((char *)enc.result, "\"vim\"") == 0);
    vim_free(enc.result);

    json_result_T dec = json_decode_rs("\"rust\"");
    assert(dec.error == NULL);
    assert(strcmp((char *)dec.result, "rust") == 0);
    vim_free(dec.result);

    json_result_T err = json_decode_rs("{invalid");
    assert(err.error != NULL);
    vim_free(err.error);
    return 0;
}
