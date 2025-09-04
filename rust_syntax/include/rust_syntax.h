#ifndef RUST_SYNTAX_H
#define RUST_SYNTAX_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

void rs_syntax_start(void *wp, long lnum);
void rs_syn_update(int startofline);
int rs_eval_line(const char *line);
void rs_add_rule(int id, const char *pattern);
void rs_clear_rules(void);

#define HL_MATCHCONT 0x8000

#ifdef __cplusplus
}
#endif

#endif // RUST_SYNTAX_H
