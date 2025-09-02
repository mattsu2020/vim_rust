#ifndef AUTOCMD_RS_H
#define AUTOCMD_RS_H

#include "vim.h"

int rs_has_autocmd(event_T event, char_u *sfname, buf_T *buf);

#endif // AUTOCMD_RS_H
