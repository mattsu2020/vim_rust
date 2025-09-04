#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

extern const uint8_t BACKSPACE;

/**
 * Equivalent of Vim's CTRL macro. Shared with C via cbindgen.
 */
uint8_t ctrl(uint8_t c);

extern void replace_join(int off);

void rs_truncate_spaces(char *line, uintptr_t len);
