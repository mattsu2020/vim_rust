#ifndef RUST_POPUPWIN_H
#define RUST_POPUPWIN_H

#include <stdint.h>

typedef struct {
    int32_t line;
    int32_t col;
    int32_t visible;
} PopupPos;

void popupwin_clear(void);
int32_t popupwin_create(const char *text, int32_t line, int32_t col);
void popupwin_close(int32_t id);
int32_t popupwin_getpos(int32_t id, PopupPos *out);
int32_t popupwin_move(int32_t id, int32_t line, int32_t col);
int32_t popupwin_show(int32_t id);
int32_t popupwin_hide(int32_t id);

#endif // RUST_POPUPWIN_H
