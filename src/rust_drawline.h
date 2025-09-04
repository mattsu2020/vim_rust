#ifndef RUST_DRAWLINE_H
#define RUST_DRAWLINE_H

#include <stdint.h>
#include "screen_rs.h"

#ifdef __cplusplus
extern "C" {
#endif

void rs_draw_line(ScreenBuffer *buf, int row, const char *text, uint8_t attr);

#ifdef __cplusplus
}
#endif

#endif // RUST_DRAWLINE_H
