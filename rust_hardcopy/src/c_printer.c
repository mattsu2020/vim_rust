#include <stdio.h>
#include "c_printer.h"

void c_print_line(const char *s) {
    if (s) {
        printf("%s\n", s);
        fflush(stdout);
    }
}
