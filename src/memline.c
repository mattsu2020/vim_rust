#include "vim.h"
#include <stdbool.h>

extern void *rs_ml_buffer_new(void);
extern void rs_ml_buffer_free(void *buf);
extern bool rs_ml_append(void *buf, size_t lnum, const char *line);
extern bool rs_ml_delete(void *buf, size_t lnum);
extern bool rs_ml_replace(void *buf, size_t lnum, const char *line);
extern unsigned char *rs_ml_get_line(void *buf, size_t lnum, int for_change, size_t *out_len);

static void *rs_buffer = NULL;

int ml_open(buf_T *buf)
{
    rs_buffer = rs_ml_buffer_new();
    // Initialize with one empty line, as Vim expects buffers to have at
    // least one line.
    (void)rs_ml_append(rs_buffer, 0, "");
    if (buf != NULL)
        buf->b_ml.ml_line_count = 1;
    return OK;
}

void ml_close(buf_T *buf UNUSED, int del_file UNUSED)
{
    if (rs_buffer != NULL)
    {
        rs_ml_buffer_free(rs_buffer);
        rs_buffer = NULL;
    }
}

int ml_append(linenr_T lnum, char_u *line, colnr_T len UNUSED, int newfile UNUSED)
{
    if (rs_ml_append(rs_buffer, (size_t)lnum, (const char *)line)) {
        // Vim uses 1-based line numbers; appending after lnum increases count by 1.
        ++curbuf->b_ml.ml_line_count;
        return OK;
    }
    return FAIL;
}

int ml_delete(linenr_T lnum)
{
    if (rs_ml_delete(rs_buffer, (size_t)lnum)) {
        if (curbuf->b_ml.ml_line_count > 0)
            --curbuf->b_ml.ml_line_count;
        return OK;
    }
    return FAIL;
}

int ml_replace(linenr_T lnum, char_u *line, int copy UNUSED)
{
    return rs_ml_replace(rs_buffer, (size_t)lnum, (const char *)line) ? OK : FAIL;
}

// Accessor to the underlying Rust buffer for use in compatibility stubs.
void *vim_rs_memline_buf(void)
{
    return rs_buffer;
}
