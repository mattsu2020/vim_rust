#ifndef RUST_EXCMD_H
#define RUST_EXCMD_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef unsigned char CharU;
typedef int GetlineOpt;
typedef int (*Fgetline)(int, void *, int, GetlineOpt);

void rs_cmd_add(const char *name, void (*func)(void));
int rs_cmd_execute(const char *name);
void rs_cmd_history_add(const char *cmd);
const char *rs_cmd_history_get(int idx);

int do_cmdline(CharU *cmdline, Fgetline fgetline, void *cookie, int flags);
CharU *do_one_cmd(CharU **cmdlinep, int flags, void *cstack, Fgetline fgetline,
                  void *cookie);
void add_bufnum(int *bufnrs, int *bufcount, int bufnum);
int rust_empty_pattern_magic(const CharU *p, size_t len, int magic_val);
int rust_should_abort(int reset);
void rust_update_force_abort(int val);
int buf_write_all(void *buf, int forceit);
void ex_listdo(void *eap);
void ex_compiler(void *eap);
void init_pyxversion(void);
void ex_pyxfile(void *eap);
void ex_pyx(void *eap);
void ex_pyxdo(void *eap);
void ex_checktime(void *eap);
char *ex_ascii(int ch);
void ex_mark_changed(void *buf);
int get_pressedreturn(void);
void set_pressedreturn(int val);

#ifdef __cplusplus
}
#endif

#endif // RUST_EXCMD_H
