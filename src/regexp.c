/* vi:set ts=8 sts=4 sw=4 noet:
 *
 * Handling of regular expressions through the Rust implementation.
 */

#include "vim.h"

// FFI bindings to Rust regex implementation
#include "../rust_regex/include/rust_regex.h"

// Exposed for Vimscript: call into Rust to match a pattern with timeout
extern int rust_regex_match(const char *pat, const char *text, int magic, long timeout_ms);

int
vim_rust_regex_match_wrapper(char_u *pat, char_u *text, int magic, long timeout_ms)
{
    return rust_regex_match((const char *)pat, (const char *)text, magic, timeout_ms);
}
