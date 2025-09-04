#ifndef RUST_CMDHIST_H
#define RUST_CMDHIST_H

#ifdef __cplusplus
extern "C" {
#endif

void rs_cmd_history_add(const char *cmd);
const char *rs_cmd_history_get(int idx);
void rs_cmd_history_init(int len);
int rs_cmd_history_len(void);
void rs_cmd_history_clear(void);

#ifdef __cplusplus
}
#endif

#endif // RUST_CMDHIST_H
