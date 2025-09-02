#ifndef SPELL_RS_H
#define SPELL_RS_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

void rs_spell_add_word(const char *word);
int rs_spell_check(const char *word);
void rs_spell_clear(void);

#ifdef __cplusplus
}
#endif

#endif // SPELL_RS_H
