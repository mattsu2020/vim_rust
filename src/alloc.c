/* vi:set ts=8 sts=4 sw=4 noet:
 *
 * VIM - Vi IMproved    by Bram Moolenaar
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

#if defined(FEAT_QUICKFIX) || defined(PROTO)
void *alloc_id(size_t size, alloc_id_T id UNUSED)
{
    return rust_alloc(size);
}
#endif

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

#if defined(FEAT_SIGNS) || defined(PROTO)
void *lalloc_id(size_t size, int message, alloc_id_T id UNUSED)
{
    return lalloc(size, message);
}
#endif

extern void *rust_mem_realloc(void *ptr, size_t size);

void *mem_realloc(void *ptr, size_t size)
{
    return rust_mem_realloc(ptr, size);
}

void vim_mem_profile_dump(void) {}
int alloc_does_fail(size_t size UNUSED) { return FALSE; }
void do_outofmem_msg(size_t size UNUSED) {}
void free_all_mem(void) {}

char_u *vim_memsave(char_u *p, size_t len)
{
    char_u *ret = alloc(len);
    if (ret != NULL)
        mch_memmove(ret, p, len);
    return ret;
}

void vim_free(void *x)
{
    rust_free(x);
}

/************************************************************************
 * Functions for handling growing arrays.
 */

/*
 * Clear an allocated growing array.
 */
    void
ga_clear(garray_T *gap)
{
    vim_free(gap->ga_data);
    ga_init(gap);
}

/*
 * Clear a growing array that contains a list of strings.
 */
    void
ga_clear_strings(garray_T *gap)
{
    int         i;

    if (gap->ga_data != NULL)
        for (i = 0; i < gap->ga_len; ++i)
            vim_free(((char_u **)(gap->ga_data))[i]);
    ga_clear(gap);
}

#if defined(FEAT_EVAL) || defined(PROTO)
/*
 * Copy a growing array that contains a list of strings.
 */
    int
ga_copy_strings(garray_T *from, garray_T *to)
{
    int         i;

    ga_init2(to, sizeof(char_u *), 1);
    if (ga_grow(to, from->ga_len) == FAIL)
        return FAIL;

    for (i = 0; i < from->ga_len; ++i)
    {
        char_u *orig = ((char_u **)from->ga_data)[i];
        char_u *copy;

        if (orig == NULL)
            copy = NULL;
        else
        {
            copy = vim_strsave(orig);
            if (copy == NULL)
            {
                to->ga_len = i;
                ga_clear_strings(to);
                return FAIL;
            }
        }
        ((char_u **)to->ga_data)[i] = copy;
    }
    to->ga_len = from->ga_len;
    return OK;
}
#endif

/*
 * Initialize a growing array.  Don't forget to set ga_itemsize and
 * ga_growsize!  Or use ga_init2().
 */
    void
ga_init(garray_T *gap)
{
    gap->ga_data = NULL;
    gap->ga_maxlen = 0;
    gap->ga_len = 0;
}

    void
ga_init2(garray_T *gap, size_t itemsize, int growsize)
{
    ga_init(gap);
    gap->ga_itemsize = (int)itemsize;
    gap->ga_growsize = growsize;
}

/*
 * Make room in growing array "gap" for at least "n" items.
 * Return FAIL for failure, OK otherwise.
 */
    int
ga_grow(garray_T *gap, int n)
{
    if (gap->ga_maxlen - gap->ga_len < n)
        return ga_grow_inner(gap, n);
    return OK;
}

/*
 * Same as ga_grow() but uses an allocation id for testing.
 */
    int
ga_grow_id(garray_T *gap, int n, alloc_id_T id UNUSED)
{
#ifdef FEAT_EVAL
    if (alloc_fail_id == id && alloc_does_fail(sizeof(list_T)))
        return FAIL;
#endif
    return ga_grow_inner(gap, n);
}

/*
 * Make room in growing array "gap" for at least "n" items.
 * Must work when gap->ga_growsize is zero.
 */
    int
ga_grow_inner(garray_T *gap, int n)
{
    int        new_len;
    char_u     *pp;
    int        old_len;

    if (n < gap->ga_growsize)
        n = gap->ga_growsize;

    if (gap->ga_maxlen - gap->ga_len < n)
    {
        // A linear growth is very inefficient when the array grows big.  This
        // is a compromise between allocating memory that won't be used and too
        // many copy operations. A factor of 1.5 seems reasonable.
        if (n < gap->ga_len / 2)
            n = gap->ga_len / 2;

        new_len = (size_t)gap->ga_itemsize * (gap->ga_len + n);
        pp = vim_realloc(gap->ga_data, new_len);
        if (pp == NULL)
            return FAIL;
        old_len = (size_t)gap->ga_itemsize * gap->ga_maxlen;
        vim_memset(pp + old_len, 0, new_len - old_len);
        gap->ga_maxlen = gap->ga_len + n;
        gap->ga_data = pp;
    }
    return OK;
}

/*
 * For a growing array that contains a list of strings: concatenate all the
 * strings with a separating "sep".
 * Returns NULL when out of memory.
 */
    char_u *
ga_concat_strings(garray_T *gap, char *sep)
{
    int         i;
    int         len = 0;
    int         sep_len = (int)STRLEN(sep);
    char_u      *s;
    char_u      *p;

    for (i = 0; i < gap->ga_len; ++i)
        len += (int)STRLEN(((char_u **)(gap->ga_data))[i]);

    if (gap->ga_len > 1)
        len += (gap->ga_len - 1) * sep_len;

    s = alloc(len + 1);
    if (s == NULL)
        return NULL;

    *s = NUL;
    p = s;
    for (i = 0; i < gap->ga_len; ++i)
    {
        if (p != s)
        {
            STRCPY(p, sep);
            p += sep_len;
        }
        STRCPY(p, ((char_u **)(gap->ga_data))[i]);
        p += STRLEN(p);
    }
    return s;
}

/*
 * Make a copy of string "p" and add it to "gap".
 * When out of memory nothing changes and FAIL is returned.
 */
    int
ga_copy_string(garray_T *gap, char_u *p)
{
    char_u *cp = vim_strsave(p);

    if (cp == NULL)
        return FAIL;

    if (ga_grow(gap, 1) == FAIL)
    {
        vim_free(cp);
        return FAIL;
    }
    ((char_u **)(gap->ga_data))[gap->ga_len++] = cp;
    return OK;
}

/*
 * Add string "p" to "gap".
 * When out of memory FAIL is returned (caller may want to free "p").
 */
    int
ga_add_string(garray_T *gap, char_u *p)
{
    if (ga_grow(gap, 1) == FAIL)
        return FAIL;
    ((char_u **)(gap->ga_data))[gap->ga_len++] = p;
    return OK;
}

/*
 * Concatenate a string to a growarray which contains bytes.
 * When "s" is NULL memory allocation fails does not do anything.
 * Note: Does NOT copy the NUL at the end!
 */
    void
ga_concat(garray_T *gap, char_u *s)
{
    int    len;

    if (s == NULL || *s == NUL)
        return;
    len = (int)STRLEN(s);
    if (ga_grow(gap, len) == OK)
    {
        mch_memmove((char *)gap->ga_data + gap->ga_len, s, (size_t)len);
        gap->ga_len += len;
    }
}

/*
 * Concatenate 'len' bytes from string 's' to a growarray.
 * When "s" is NULL does not do anything.
 */
    void
ga_concat_len(garray_T *gap, char_u *s, size_t len)
{
    if (s == NULL || *s == NUL || len == 0)
        return;
    if (ga_grow(gap, (int)len) == OK)
    {
        mch_memmove((char *)gap->ga_data + gap->ga_len, s, len);
        gap->ga_len += (int)len;
    }
}

/*
 * Append one byte to a growarray which contains bytes.
 */
    int
ga_append(garray_T *gap, int c)
{
    if (ga_grow(gap, 1) == FAIL)
        return FAIL;
    ((char *)gap->ga_data)[gap->ga_len++] = c;
    return OK;
}

/*
 * Append a NUL terminated string to a growarray and add the terminating NUL.
 */
    void
append_ga_line(garray_T *gap)
{
    if (ga_grow(gap, gap->ga_len + 1) == OK)
    {
        ((char *)gap->ga_data)[gap->ga_len] = NUL;
        ++gap->ga_len;
    }
}

/* vim: set ft=c : */
