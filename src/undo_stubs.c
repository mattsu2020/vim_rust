// Minimal stub implementations for legacy undo API
// to satisfy linker when building this configuration.
// These do not provide real undo functionality.

#include "vim.h"

int u_save_cursor(void) { return OK; }
int u_save(linenr_T top, linenr_T bot) { (void)top; (void)bot; return OK; }
int u_savesub(linenr_T lnum) { (void)lnum; return OK; }
int u_savecommon(linenr_T top, linenr_T bot, linenr_T newbot, int reload)
{ (void)top; (void)bot; (void)newbot; (void)reload; return OK; }

void u_compute_hash(char_u *hash) { (void)hash; }
void u_write_undo(char_u *name, int forceit, buf_T *buf, char_u *hash)
{ (void)name; (void)forceit; (void)buf; (void)hash; }
void u_read_undo(char_u *name, char_u *hash, char_u *fname)
{ (void)name; (void)hash; (void)fname; }

void u_undo(int count) { (void)count; }
void u_redo(int count) { (void)count; }
void u_undoline(void) {}
void u_undofile_reset_and_delete(buf_T *buf) { (void)buf; }
void u_update_save_nr(buf_T *buf) { (void)buf; }
void u_unchanged(buf_T *buf) { (void)buf; }
void u_find_first_changed(void) {}
void u_clearline(void) {}

int u_inssub(linenr_T lnum) { (void)lnum; return OK; }
int u_savedel(linenr_T lnum, long nlines) { (void)lnum; (void)nlines; return OK; }
int undo_allowed(void) { return TRUE; }
void undo_time(long step, int sec, int file, int absolute)
{ (void)step; (void)sec; (void)file; (void)absolute; }
void u_sync(int force) { (void)force; }
void ex_undolist(exarg_T *eap) { (void)eap; }
void ex_undojoin(exarg_T *eap) { (void)eap; }
int bufIsChanged(buf_T *buf) { (void)buf; return FALSE; }
int anyBufIsChanged(void) { return FALSE; }
int bufIsChangedNotTerm(buf_T *buf) { (void)buf; return FALSE; }
int curbufIsChanged(void) { return FALSE; }
void f_undofile(typval_T *argvars, typval_T *rettv)
{ (void)argvars; if (rettv) rettv->vval.v_number = 0; }
void f_undotree(typval_T *argvars, typval_T *rettv)
{ (void)argvars; if (rettv) rettv->vval.v_number = 0; }
void undo_cmdmod(cmdmod_T *cmod) { (void)cmod; }
void u_clearallandblockfree(buf_T *buf) { (void)buf; }
