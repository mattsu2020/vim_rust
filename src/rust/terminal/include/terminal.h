#ifndef TERMINAL_RS_H
#define TERMINAL_RS_H

#ifdef __cplusplus
extern "C" {
#endif

int term_start(void);
int term_write(const char *data);
int term_stop(void);

#ifdef __cplusplus
}
#endif

#endif // TERMINAL_RS_H
