#ifndef RUST_MOUSE_H
#define RUST_MOUSE_H

#include <stdint.h>

int rs_handle_mouse_event(void *oap, int c, int dir, long count, int fixindent);

#endif // RUST_MOUSE_H
