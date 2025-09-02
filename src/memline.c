#include "vim.h"
#include <stdbool.h>

extern void *rs_ml_buffer_new(void);
extern void rs_ml_buffer_free(void *buf);
extern bool rs_ml_append(void *buf, size_t lnum, const char *line);
extern bool rs_ml_delete(void *buf, size_t lnum);
extern bool rs_ml_replace(void *buf, size_t lnum, const char *line);

static void *rs_buffer = NULL;

int ml_open(buf_T *buf)
{
    rs_buffer = rs_ml_buffer_new();
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
    return rs_ml_append(rs_buffer, (size_t)lnum, (const char *)line) ? OK : FAIL;
}

int ml_delete(linenr_T lnum)
{
    return rs_ml_delete(rs_buffer, (size_t)lnum) ? OK : FAIL;
}

int ml_replace(linenr_T lnum, char_u *line, int copy UNUSED)
{
    return rs_ml_replace(rs_buffer, (size_t)lnum, (const char *)line) ? OK : FAIL;
}
