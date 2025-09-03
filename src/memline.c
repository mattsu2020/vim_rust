#include "vim.h"
#include <stdbool.h>

extern void *rs_ml_buffer_new(void);
extern void rs_ml_buffer_free(void *buf);
extern bool rs_ml_append(void *buf, size_t lnum, const char *line);
extern bool rs_ml_delete(void *buf, size_t lnum);
extern bool rs_ml_replace(void *buf, size_t lnum, const char *line);
extern unsigned char *rs_ml_get_line(void *buf, size_t lnum, int for_change, size_t *out_len);

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
