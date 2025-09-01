/* vi:set ts=8 sts=4 sw=4 noet:
 *
 * VIM - Vi IMproved	by Bram Moolenaar
 *
 * Do ":help uganda"  in Vim to read copying and usage conditions.
 * Do ":help credits" in Vim to see a list of people who contributed.
 * See README.txt for an overview of the Vim source code.
 */

/*
 * alloc.c: functions for memory management
 */

#include "vim.h"


// Declarations for the Rust allocation implementation.
extern void *rust_alloc(size_t size);
extern void *rust_alloc_clear(size_t size);
extern void rust_free(void *ptr);

void *alloc(size_t size)
{
    return rust_alloc(size);
}

# if defined(FEAT_QUICKFIX) || defined(PROTO)
void *alloc_id(size_t size, alloc_id_T id UNUSED)
{
    return rust_alloc(size);
}
# endif

void *alloc_clear(size_t size)
{
    return rust_alloc_clear(size);
}

void *alloc_clear_id(size_t size, alloc_id_T id UNUSED)
{
    return rust_alloc_clear(size);
}

void *lalloc_clear(size_t size, int message)
{
    (void)message;
    return rust_alloc_clear(size);
}

void *lalloc(size_t size, int message)
{
    (void)message;
    return rust_alloc(size);
}

# if defined(FEAT_SIGNS) || defined(PROTO)
void *lalloc_id(size_t size, int message, alloc_id_T id UNUSED)
{
    return lalloc(size, message);
}
# endif

void vim_free(void *x)
{
    rust_free(x);
}
