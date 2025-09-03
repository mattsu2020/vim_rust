/* message.c (Rust replacement) */
/* prototypes provided by rust_message crate */
char *rs_pop_message(int *level);
void rs_queue_message(char *msg, int level);
char *rs_get_last_error(void);
void rs_clear_messages(void);
void rs_free_cstring(char *s);
/* vim: set ft=c : */
