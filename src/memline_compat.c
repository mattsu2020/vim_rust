// Compatibility layer wired to Rust-backed memline implementation.

#include "vim.h"


void ml_set_crypt_key(buf_T *buf, char_u *old_key, char_u *old_cm)
{ (void)buf; (void)old_key; (void)old_cm; }
void ml_setname(buf_T *buf) { (void)buf; }
void ml_open_files(void) {}
void ml_open_file(buf_T *buf) { (void)buf; }
void check_need_swap(int newfile) { (void)newfile; }
void ml_close_all(int del_file) { (void)del_file; }
void ml_close_notmod(void) {}
void ml_timestamp(buf_T *buf) { (void)buf; }
void ml_recover(int checkext) { (void)checkext; }
int recover_names(char_u *fname, int do_list, list_T *ret_list, int nr, char_u **fname_out)
{ (void)fname; (void)do_list; (void)ret_list; (void)nr; (void)fname_out; return FAIL; }
char_u *make_percent_swname(char_u *dir, char_u *dir_end, char_u *name)
{ (void)dir; (void)dir_end; (void)name; return NULL; }
void get_b0_dict(char_u *fname, dict_T *d) { (void)fname; (void)d; }
void ml_sync_all(int check_file, int check_char) { (void)check_file; (void)check_char; }
void ml_preserve(buf_T *buf, int message) { (void)buf; (void)message; }


void ml_setmarked(linenr_T lnum) { (void)lnum; }
linenr_T ml_firstmarked(void) { return 0; }
void ml_clearmarked(void) {}
int resolve_symlink(char_u *fname, char_u *buf) { (void)fname; (void)buf; return FAIL; }
char_u *makeswapname(char_u *fname, char_u *ffname, buf_T *buf, char_u *dir_name)
{ (void)fname; (void)ffname; (void)buf; (void)dir_name; return NULL; }
char_u *get_file_in_dir(char_u *fname, char_u *dname)
{ (void)fname; (void)dname; return NULL; }
void ml_setflags(buf_T *buf) { (void)buf; }
char_u *ml_encrypt_data(memfile_T *mfp, char_u *data, off_T offset, unsigned size)
{ (void)mfp; (void)data; (void)offset; (void)size; return NULL; }
void ml_decrypt_data(memfile_T *mfp, char_u *data, off_T offset, unsigned size)
{ (void)mfp; (void)data; (void)offset; (void)size; }
long ml_find_line_or_offset(buf_T *buf, linenr_T lnum, long *offp)
{ (void)buf; (void)lnum; (void)offp; return 0; }
void goto_byte(long cnt) { (void)cnt; }
