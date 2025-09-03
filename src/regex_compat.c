// Minimal compatibility wrappers for legacy Vim regex APIs when using
// the Rust regex engine. These are intentionally simplistic to unblock
// compilation; behavior is limited.

#include "vim.h"

#ifdef USE_RUST_REGEX

// Minimal helpers to keep APIs that used to live in regexp.c available when
// USE_RUST_REGEX is enabled.

// Scan forward until encountering delim or NUL, honoring simple escapes and
// bracketed character classes. This is not a full parser, but good enough for
// callers that only want to skip over a pattern.
static char_u *scan_until_delim(char_u *p, int delim)
{
    while (p[0] != NUL)
    {
        if (p[0] == (char)delim)
            break;
        if (p[0] == '\\' && p[1] != NUL)
        {
            // skip escaped char
            p += 2;
            continue;
        }
        if (p[0] == '[')
        {
            // skip simple [...] class
            ++p;
            while (p[0] != NUL && p[0] != ']')
            {
                if (p[0] == '\\' && p[1] != NUL)
                    p += 2;
                else
                    ++p;
            }
            if (p[0] == ']')
                ++p;
            continue;
        }
        ++p;
    }
    return p;
}

char_u *skip_regexp(char_u *startp, int delim, int magic)
{
    (void)magic; // ignored in this simplified implementation
    return scan_until_delim(startp, delim);
}

char_u *skip_regexp_err(char_u *startp, int delim, int magic)
{
    char_u *p = skip_regexp(startp, delim, magic);
    if (*p != (char)delim)
        return NULL;
    return p;
}

char_u *skip_regexp_ex(
    char_u *startp,
    int dirc,
    int magic,
    char_u **newp,
    int *dropped,
    magic_T *magic_val)
{
    if (newp != NULL)
        *newp = NULL;
    if (dropped != NULL)
        *dropped = 0;
    if (magic_val != NULL)
        *magic_val = magic ? MAGIC_ON : MAGIC_OFF;
    return skip_regexp(startp, dirc, magic);
}

void unref_extmatch(reg_extmatch_T *em)
{
    if (em == NULL)
        return;
    if (--em->refcnt <= 0)
    {
        for (int i = 0; i < NSUBEXP; ++i)
            vim_free(em->matches[i]);
        vim_free(em);
    }
}

// Minimal wrapper for callers expecting this entry point.
int vim_regexec_prog(regprog_T **prog, int ignore_case, char_u *line, colnr_T col)
{
    (void)prog; (void)ignore_case; (void)line; (void)col;
    return 0;
}

int vim_regcomp_had_eol(void)
{
    return 0;
}

// Debug-timeout helpers used from other modules.
void save_timeout_for_debugging(void) {}
void restore_timeout_for_debugging(void) {}

// Stubs for regex APIs used elsewhere when using the Rust engine.
void init_regexp_timeout(long msec) { (void)msec; }
void disable_regexp_timeout(void) {}
int re_multiline(regprog_T *prog)
{
    (void)prog; return 0;
}

reg_extmatch_T *ref_extmatch(reg_extmatch_T *em)
{
    if (em != NULL)
        em->refcnt++;
    return em;
}

char_u *regtilde(char_u *source, int magic)
{
    (void)magic; return source;
}

void free_resub_eval_result(void) {}

char_u *reg_submatch(int no)
{
    (void)no; static char_u empty[] = ""; return empty;
}

list_T *reg_submatch_list(int no)
{
    (void)no; return NULL;
}

// Legacy multi-line variant. Minimal no-op.
int vim_regsub_multi(regmmatch_T *rmp, linenr_T lnum, char_u *source,
                     char_u *dest, int destlen, int flags)
{
    (void)rmp; (void)lnum; (void)source; (void)flags;
    if (dest != NULL && destlen > 0)
        dest[0] = NUL;
    return 0;
}

#endif // USE_RUST_REGEX
