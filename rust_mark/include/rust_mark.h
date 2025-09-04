#include <stdbool.h>
#include <stdint.h>

typedef struct {
    int64_t line;
    int64_t col;
} Position;

void mark_set(char name, int64_t line, int64_t col);
bool mark_get(char name, Position *out);
void mark_clear(char name);
