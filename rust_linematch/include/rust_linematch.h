#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct {
    const char *ptr;
    size_t size;
} MMFile;

int matching_chars(const MMFile *m1, const MMFile *m2);
int matching_chars_ignore_whitespace(const MMFile *m1, const MMFile *m2);
