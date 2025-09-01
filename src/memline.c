#include "vim.h"
#include "memline_ffi.h"

static MemBuffer *g_mem = NULL;

int ml_open(buf_T *buf)
{
    (void)buf;
    g_mem = rs_ml_buffer_new();
    return g_mem != NULL ? OK : FAIL;
}

void ml_close(buf_T *buf, int del_file)
{
    (void)buf;
    (void)del_file;
    if (g_mem != NULL)
    {
        rs_ml_buffer_free(g_mem);
        g_mem = NULL;
    }
}

char_u *ml_get(linenr_T lnum)
{
    const char *p = rs_ml_get_line(g_mem, (size_t)lnum);
    return (char_u *)p;
}

int ml_append(linenr_T lnum, char_u *line, colnr_T len, int newfile)
{
    (void)len;
    (void)newfile;
    return rs_ml_append(g_mem, (size_t)lnum, (const char *)line);
}

int ml_delete(linenr_T lnum)
{
    return rs_ml_delete(g_mem, (size_t)lnum);
}

int ml_replace(linenr_T lnum, char_u *line, int copy)
{
    (void)copy;
    return rs_ml_replace(g_mem, (size_t)lnum, (const char *)line);
}
