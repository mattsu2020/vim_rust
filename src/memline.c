#include "vim.h"
#include <stdbool.h>
#include "rust_memline.h"

static char_u empty_line[] = "";

int ml_open(buf_T *buf)
{
    // Allocate a fresh Rust memline buffer and store the opaque pointer in
    // b_ml.ml_mfp.  The rest of the memline_T fields are unused by the Rust
    // backend but keeping them allows existing code to keep working.
    buf->b_ml.ml_mfp = (memfile_T *)ml_buffer_new();
    if (buf->b_ml.ml_mfp == NULL)
        return FAIL;

    // Initialize with one empty line, as Vim expects buffers to have at least
    // one line.
    (void)ml_append((MemBuffer *)buf->b_ml.ml_mfp, 0, "");
    buf->b_ml.ml_line_count = (linenr_T)ml_line_count((MemBuffer *)buf->b_ml.ml_mfp);
    return OK;
}

void ml_close(buf_T *buf, int del_file UNUSED)
{
    if (buf != NULL && buf->b_ml.ml_mfp != NULL)
    {
        ml_buffer_free((MemBuffer *)buf->b_ml.ml_mfp);
        buf->b_ml.ml_mfp = NULL;
    }
}

int ml_append(linenr_T lnum, char_u *line, colnr_T len UNUSED, int newfile UNUSED)
{
    MemBuffer *rs_buffer = (MemBuffer *)curbuf->b_ml.ml_mfp;
    if (!ml_append(rs_buffer, (size_t)lnum, (const char *)line))
        return FAIL;
    curbuf->b_ml.ml_line_count = (linenr_T)ml_line_count(rs_buffer);
    return OK;
}

int ml_delete(linenr_T lnum)
{
    MemBuffer *rs_buffer = (MemBuffer *)curbuf->b_ml.ml_mfp;
    if (!ml_delete(rs_buffer, (size_t)lnum))
        return FAIL;
    curbuf->b_ml.ml_line_count = (linenr_T)ml_line_count(rs_buffer);
    return OK;
}

int ml_replace(linenr_T lnum, char_u *line, int copy UNUSED)
{
    MemBuffer *rs_buffer = (MemBuffer *)curbuf->b_ml.ml_mfp;
    return ml_replace(rs_buffer, (size_t)lnum, (const char *)line) ? OK : FAIL;
}

char_u *ml_get(linenr_T lnum)
{
    size_t len = 0;
    char_u *p = (char_u *)ml_get_line((MemBuffer *)curbuf->b_ml.ml_mfp, (size_t)lnum, 0, &len);
    curbuf->b_ml.ml_line_ptr = p;
    curbuf->b_ml.ml_line_lnum = lnum;
    curbuf->b_ml.ml_line_len = (colnr_T)(len + 1);
    curbuf->b_ml.ml_line_textlen = (colnr_T)(len + 1);
    curbuf->b_ml.ml_flags &= ~ML_ALLOCATED;
    return p;
}

char_u *ml_get_pos(pos_T *pos)
{
    if (pos == NULL) return empty_line;
    return ml_get(pos->lnum);
}

char_u *ml_get_curline(void)
{
    return ml_get(curbuf->b_ml.ml_line_count > 0 ? curwin->w_cursor.lnum : 1);
}

char_u *ml_get_cursor(void)
{
    return ml_get(curwin->w_cursor.lnum);
}

colnr_T ml_get_len(linenr_T lnum)
{
    size_t len = 0;
    (void)ml_get_line((MemBuffer *)curbuf->b_ml.ml_mfp, (size_t)lnum, 0, &len);
    return (colnr_T)len;
}

colnr_T ml_get_pos_len(pos_T *pos)
{
    if (pos == NULL) return 0;
    return ml_get_len(pos->lnum);
}

colnr_T ml_get_curline_len(void)
{
    return ml_get_len(curwin->w_cursor.lnum);
}

colnr_T ml_get_cursor_len(void)
{
    return ml_get_len(curwin->w_cursor.lnum);
}

colnr_T ml_get_buf_len(buf_T *buf, linenr_T lnum)
{
    size_t len = 0;
    (void)ml_get_line((MemBuffer *)buf->b_ml.ml_mfp, (size_t)lnum, 0, &len);
    return (colnr_T)len;
}

char_u *ml_get_buf(buf_T *buf, linenr_T lnum, int will_change)
{
    size_t len = 0;
    char_u *p = (char_u *)ml_get_line((MemBuffer *)buf->b_ml.ml_mfp, (size_t)lnum, will_change ? 1 : 0, &len);
    curbuf->b_ml.ml_line_ptr = p;
    curbuf->b_ml.ml_line_lnum = lnum;
    curbuf->b_ml.ml_line_len = (colnr_T)(len + 1);
    curbuf->b_ml.ml_line_textlen = (colnr_T)(len + 1);
    if (will_change)
        curbuf->b_ml.ml_flags |= ML_ALLOCATED;
    else
        curbuf->b_ml.ml_flags &= ~ML_ALLOCATED;
    return p;
}

int ml_line_alloced(void) { return (curbuf->b_ml.ml_flags & ML_ALLOCATED) != 0; }

int ml_append_flags(linenr_T lnum, char_u *line, colnr_T len, int flags)
{
    (void)lnum; (void)flags; (void)len; return ml_append(lnum, line, len, FALSE);
}

int ml_append_buf(buf_T *buf, linenr_T lnum, char_u *line, colnr_T len, int newfile)
{
    (void)buf; return ml_append(lnum, line, len, newfile);
}

int ml_replace_len(linenr_T lnum, char_u *line_arg, colnr_T len_arg, int has_props, int copy)
{
    (void)has_props; (void)copy; (void)len_arg; return ml_replace(lnum, line_arg, FALSE);
}

int ml_delete_flags(linenr_T lnum, int flags)
{
    (void)flags; return ml_delete(lnum);
}
