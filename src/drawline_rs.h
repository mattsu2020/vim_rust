#ifndef DRAWLINE_RS_H
#define DRAWLINE_RS_H

#include "screen_rs.h"
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int rs_draw_line(ScreenBuffer *buf, int row, const char *line);

#ifdef __cplusplus
}
#endif

#endif // DRAWLINE_RS_H
