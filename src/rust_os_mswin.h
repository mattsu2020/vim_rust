#ifndef RUST_OS_MSWIN_H
#define RUST_OS_MSWIN_H

#include <stdint.h>

void os_mswin_startup(void);
void os_mswin_shutdown(void);
uint32_t os_mswin_get_tick_count(void);
int os_mswin_isatty(int fd);

#endif // RUST_OS_MSWIN_H
