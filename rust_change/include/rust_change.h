#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Equivalent of Vim's change_warning().  Returns 1 when a warning should be
 * displayed and 0 otherwise.  Only warns once while the buffer is marked as
 * readonly and not yet changed.
 */
int change_warning(int _col);

/**
 * Mark the buffer as changed.  This mirrors Vim's changed() which notifies
 * the editor that modifications have been made.
 */
void changed(void);
