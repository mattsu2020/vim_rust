#ifndef DRAWSCREEN_RS_H
#define DRAWSCREEN_RS_H

#include <stdint.h>
#include "screen_rs.h"

#ifdef __cplusplus
extern "C" {
#endif

void rs_drawscreen_init(ScreenBuffer *buf, int width, int height);
void rs_update_screen(int type_arg);

#ifdef __cplusplus
}
#endif

#endif // DRAWSCREEN_RS_H
