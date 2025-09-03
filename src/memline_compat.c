// Compatibility layer wired to Rust-backed memline implementation.

#include "vim.h"

extern unsigned char *rs_ml_get_line(void *buf, size_t lnum, int for_change, size_t *out_len);

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

static char_u empty_line[] = "";

char_u *ml_get(linenr_T lnum)
{
    size_t len = 0;
    char_u *p = (char_u *)rs_ml_get_line((void *)curbuf->b_ml.ml_mfp, (size_t)lnum, 0, &len);
    curbuf->b_ml.ml_line_ptr = p;
    curbuf->b_ml.ml_line_lnum = lnum;
    curbuf->b_ml.ml_line_len = (colnr_T)(len + 1);
    curbuf->b_ml.ml_line_textlen = (colnr_T)(len + 1);
    // 読み取り用途では ML_ALLOCATED は立てない
    curbuf->b_ml.ml_flags &= ~ML_ALLOCATED;
    return p;
}

char_u *ml_get_pos(pos_T *pos)
{
    if (pos == NULL) return empty_line;
    return ml_get(pos->lnum);
}

char_u *ml_get_curline(void)
{
    return ml_get(curbuf->b_ml.ml_line_count > 0 ? curwin->w_cursor.lnum : 1);
}

char_u *ml_get_cursor(void)
{
    return ml_get(curwin->w_cursor.lnum);
}

colnr_T ml_get_len(linenr_T lnum)
{
    size_t len = 0;
    (void)rs_ml_get_line((void *)curbuf->b_ml.ml_mfp, (size_t)lnum, 0, &len);
    return (colnr_T)len;
}

colnr_T ml_get_pos_len(pos_T *pos)
{
    if (pos == NULL) return 0;
    return ml_get_len(pos->lnum);
}

colnr_T ml_get_curline_len(void)
{
    return ml_get_len(curwin->w_cursor.lnum);
}

colnr_T ml_get_cursor_len(void)
{
    return ml_get_len(curwin->w_cursor.lnum);
}

colnr_T ml_get_buf_len(buf_T *buf, linenr_T lnum)
{
    size_t len = 0;
    (void)rs_ml_get_line((void *)buf->b_ml.ml_mfp, (size_t)lnum, 0, &len);
    return (colnr_T)len;
}

char_u *ml_get_buf(buf_T *buf, linenr_T lnum, int will_change)
{
    size_t len = 0;
    char_u *p = (char_u *)rs_ml_get_line((void *)buf->b_ml.ml_mfp, (size_t)lnum, will_change ? 1 : 0, &len);
    curbuf->b_ml.ml_line_ptr = p;
    curbuf->b_ml.ml_line_lnum = lnum;
    curbuf->b_ml.ml_line_len = (colnr_T)(len + 1);
    curbuf->b_ml.ml_line_textlen = (colnr_T)(len + 1);
    if (will_change)
        curbuf->b_ml.ml_flags |= ML_ALLOCATED;
    else
        curbuf->b_ml.ml_flags &= ~ML_ALLOCATED;
    return p;
}

int ml_line_alloced(void) { return (curbuf->b_ml.ml_flags & ML_ALLOCATED) != 0; }

int ml_append_flags(linenr_T lnum, char_u *line, colnr_T len, int flags)
{ (void)lnum; (void)flags; (void)len; return ml_append(lnum, line, len, FALSE); }

int ml_append_buf(buf_T *buf, linenr_T lnum, char_u *line, colnr_T len, int newfile)
{ (void)buf; return ml_append(lnum, line, len, newfile); }

int ml_replace_len(linenr_T lnum, char_u *line_arg, colnr_T len_arg, int has_props, int copy)
{ (void)has_props; (void)copy; (void)len_arg; return ml_replace(lnum, line_arg, FALSE); }

int ml_delete_flags(linenr_T lnum, int flags)
{ (void)flags; return ml_delete(lnum); }

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
