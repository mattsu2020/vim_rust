#ifndef SCREEN_RS_H
#define SCREEN_RS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ScreenBuffer ScreenBuffer;

ScreenBuffer *rs_screen_new(int width, int height);
void rs_screen_free(ScreenBuffer *buf);
void rs_screen_draw_text(ScreenBuffer *buf, int row, int col, const char *text, uint8_t attr);
void rs_screen_clear_line(ScreenBuffer *buf, int row, uint8_t attr);
void rs_screen_clear(ScreenBuffer *buf, uint8_t attr);
void rs_screen_highlight(ScreenBuffer *buf, int row, int col, int len, uint8_t attr);
typedef void (*rs_flush_cb)(int row, const char *text, const uint8_t *attr, int len);
void rs_screen_flush(ScreenBuffer *buf, rs_flush_cb cb);

#ifdef __cplusplus
}
#endif

#endif // SCREEN_RS_H
