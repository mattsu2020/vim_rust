#ifndef RUST_TAG_H
#define RUST_TAG_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int rust_find_tags(const char *pat, int *num_matches, char ***matchesp,
                   int flags, int mincount, const char *buf_ffname);
void rust_tagstack_push(const char *tag);
char *rust_tagstack_pop(void);
int rust_tagstack_len(void);

#ifdef __cplusplus
}
#endif

#endif // RUST_TAG_H
