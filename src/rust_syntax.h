#ifndef RUST_SYNTAX_H
#define RUST_SYNTAX_H

#include <stdint.h>

void rs_syntax_start(void *wp, long lnum);
void rs_syn_update(int startofline);

#endif // RUST_SYNTAX_H
