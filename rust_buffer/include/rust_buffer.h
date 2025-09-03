#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

void *buf_alloc(size_t size);
void buf_free(void *buf);
void buf_freeall(void *buf, int flags);

int calc_percentage(long part, long whole);
int get_highest_fnum(void);
int get_buf_free_count(void);
void inc_buf_free_count(void);

int get_top_file_num(void);
void set_top_file_num(int num);
int next_top_file_num(void);
void dec_top_file_num(void);

typedef struct {
    void *br_buf;
    int br_fnum;
    int br_buf_free_count;
} bufref_T;

void set_bufref(bufref_T *bufref, void *buf);
int bufref_valid(const bufref_T *bufref);
int buf_valid(void *buf);
