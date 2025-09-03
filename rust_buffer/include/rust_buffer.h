#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

void *buf_alloc(size_t size);
void buf_free(void *buf);
void buf_freeall(void *buf, int flags);

int calc_percentage(long part, long whole);
int get_highest_fnum(void);

int get_top_file_num(void);
void set_top_file_num(int num);
int next_top_file_num(void);
void dec_top_file_num(void);
