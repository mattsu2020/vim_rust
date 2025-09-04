#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct Regex Regex;

Regex *regex_new(const char *pattern);
bool regex_is_match(const Regex *re, const char *text);
void regex_free(Regex *re);
