#include <stdlib.h>
#include "rust/terminal_full/include/terminal_full.h"

// Simplified C shim delegating terminal operations to the Rust implementation.
// This is a drastic reduction from the original terminal.c which contained the
// full terminal emulator.  Only a few entry points are kept as thin wrappers.

typedef struct terminal {
    Terminal *rust;
} terminal_T;

terminal_T *term_new(int cols, int rows) {
    terminal_T *t = (terminal_T *)malloc(sizeof(terminal_T));
    if (t == NULL) {
        return NULL;
    }
    t->rust = terminal_full_new(cols, rows);
    return t;
}

void term_free(terminal_T *t) {
    if (t == NULL) {
        return;
    }
    terminal_full_free(t->rust);
    free(t);
}

void term_write(terminal_T *t, const char *data) {
    if (t == NULL) {
        return;
    }
    terminal_full_write(t->rust, data);
}

