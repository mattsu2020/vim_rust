#ifndef RUST_SPELL_H
#define RUST_SPELL_H

#include <stdbool.h>

bool rs_spell_load_dictionary(const char *path);
bool rs_spell_check(const char *word);
char *rs_spell_best_suggestion(const char *word);
void rs_spell_free_cstring(char *s);

#endif // RUST_SPELL_H

