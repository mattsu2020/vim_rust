/* Regular expression logic moved to Rust implementation via FFI.
 * This C file now only provides minimal stubs and wrappers.
 */

#include "vim.h"
#include "regexp.h"

#ifdef USE_RUST_REGEX

void init_regexp_timeout(long msec) { (void)msec; }
void disable_regexp_timeout(void) {}
void save_timeout_for_debugging(void) {}
void restore_timeout_for_debugging(void) {}
int re_multiline(regprog_T *prog) { (void)prog; return 0; }
int vim_regcomp_had_eol(void) { return 0; }
int regprog_in_use(regprog_T *prog) { (void)prog; return 0; }
int vim_regexec_prog(regprog_T **prog, int ignore_case, char_u *line, colnr_T col)
{
    regmatch_T rm;
    rm.regprog = *prog;
    rm.rm_ic = ignore_case;
    return vim_regexec(&rm, line, col);
}

#else
#error "Rust regex engine required"
#endif
