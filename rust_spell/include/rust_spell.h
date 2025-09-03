#ifndef RUST_SPELL_H
#define RUST_SPELL_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

int captype(const unsigned char *word, const unsigned char *end);
bool rs_spell_load_dict(const char *path);
bool rs_spell_check(const char *word);
char **rs_spell_suggest(const char *word, size_t max, size_t *len);
void rs_spell_free_suggestions(char **ptr, size_t len);

#endif // RUST_SPELL_H
