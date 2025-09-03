#include "vim.h"
#include <stdbool.h>

extern void *rs_ml_buffer_new(void);
extern void rs_ml_buffer_free(void *buf);
extern bool rs_ml_append(void *buf, size_t lnum, const char *line);
extern bool rs_ml_delete(void *buf, size_t lnum);
extern bool rs_ml_replace(void *buf, size_t lnum, const char *line);
extern unsigned char *rs_ml_get_line(void *buf, size_t lnum, int for_change, size_t *out_len);

static char_u empty_line[] = "";

int ml_open(buf_T *buf)
{
    // Allocate a fresh Rust memline buffer and store the opaque pointer in
    // b_ml.ml_mfp.  The rest of the memline_T fields are unused by the Rust
    // backend but keeping them allows existing code to keep working.
    buf->b_ml.ml_mfp = (memfile_T *)rs_ml_buffer_new();
    if (buf->b_ml.ml_mfp == NULL)
        return FAIL;

    // Initialize with one empty line, as Vim expects buffers to have at least
    // one line.
    (void)rs_ml_append((void *)buf->b_ml.ml_mfp, 0, "");
    buf->b_ml.ml_line_count = 1;
    return OK;
}

void ml_close(buf_T *buf, int del_file UNUSED)
{
    if (buf != NULL && buf->b_ml.ml_mfp != NULL)
    {
        rs_ml_buffer_free((void *)buf->b_ml.ml_mfp);
        buf->b_ml.ml_mfp = NULL;
    }
}

int ml_append(linenr_T lnum, char_u *line, colnr_T len UNUSED, int newfile UNUSED)
{
    void *rs_buffer = (void *)curbuf->b_ml.ml_mfp;
    if (rs_ml_append(rs_buffer, (size_t)lnum, (const char *)line)) {
        // Vim uses 1-based line numbers; appending after lnum increases count
        // by 1.
        ++curbuf->b_ml.ml_line_count;
        return OK;
    }
    return FAIL;
}

int ml_delete(linenr_T lnum)
{
    void *rs_buffer = (void *)curbuf->b_ml.ml_mfp;
    if (rs_ml_delete(rs_buffer, (size_t)lnum)) {
        if (curbuf->b_ml.ml_line_count > 0)
            --curbuf->b_ml.ml_line_count;
        return OK;
    }
    return FAIL;
}

int ml_replace(linenr_T lnum, char_u *line, int copy UNUSED)
{
    void *rs_buffer = (void *)curbuf->b_ml.ml_mfp;
    return rs_ml_replace(rs_buffer, (size_t)lnum, (const char *)line) ? OK : FAIL;
}

char_u *ml_get(linenr_T lnum)
{
    size_t len = 0;
    char_u *p = (char_u *)rs_ml_get_line((void *)curbuf->b_ml.ml_mfp, (size_t)lnum, 0, &len);
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
    (void)rs_ml_get_line((void *)curbuf->b_ml.ml_mfp, (size_t)lnum, 0, &len);
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
    (void)rs_ml_get_line((void *)buf->b_ml.ml_mfp, (size_t)lnum, 0, &len);
    return (colnr_T)len;
}

char_u *ml_get_buf(buf_T *buf, linenr_T lnum, int will_change)
{
    size_t len = 0;
    char_u *p = (char_u *)rs_ml_get_line((void *)buf->b_ml.ml_mfp, (size_t)lnum, will_change ? 1 : 0, &len);
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
