// Minimal compatibility for functions usually provided by ex_docmd.c
#include "vim.h"

int vim_mkdir_emsg(char_u *name, int prot)
{
    return vim_mkdir(name, prot) == OK ? OK : FAIL;
}

// Turn 'hlsearch' highlighting on/off.
void set_no_hlsearch(int flag)
{
#ifdef FEAT_SEARCH_EXTRA
    no_hlsearch = flag ? TRUE : FALSE;
#else
    (void)flag;
#endif
}

// Request a redraw of given kind.
void set_must_redraw(int type)
{
    if (type > must_redraw)
        must_redraw = type;
}

// Not implemented: show the ruler in the status line.
void showruler(int always)
{
    (void)always;
}

// pressedreturn は rust_excmd が提供するため未定義にしておく

// Parse helpers (very simplified)
char_u *skip_option_env_lead(char_u *start)
{
    return start;
}

char_u *skip_range(char_u *cmd_start, int skip_star, int *ctx)
{
    (void)skip_star; (void)ctx;
    // Very basic: skip digits, marks and separators :,;.-+$% 
    char_u *p = cmd_start;
    while (*p && (vim_isdigit(*p) || *p == ':' || *p == ',' || *p == ';'
                || *p == '.' || *p == '-' || *p == '+' || *p == '$'
                || *p == '%' || *p == '\''))
        ++p;
    return p;
}

char_u *skip_cmd_arg(char_u *p, int rembs)
{
    (void)rembs;
    while (*p != NUL && *p != ' ' && *p != '\t' && *p != '|')
    {
        if (*p == '\\' && p[1] != NUL)
            p += 2;
        else
            ++p;
    }
    return p;
}

void separate_nextcmd(exarg_T *eap, int keep_backslash)
{
    (void)eap; (void)keep_backslash;
}

void set_nextcmd(exarg_T *eap, char_u *arg)
{
    (void)eap; (void)arg;
}

void apply_cmdmod(cmdmod_T *cmod)
{
    (void)cmod;
}

int before_quit_all(exarg_T *eap)
{
    (void)eap;
    return OK;
}

char_u *check_nextcmd(char_u *p)
{
    return p;
}

void trigger_DirChangedPre(char_u *acmd_fname, char_u *new_dir)
{
    (void)acmd_fname; (void)new_dir;
}

int changedir_func(char_u *new_dir, int forceit, cdscope_T scope)
{
    (void)new_dir; (void)forceit; (void)scope;
    return OK;
}

void tabpage_new(void)
{
}

int set_ref_in_findfunc(int copyID)
{
    (void)copyID; return 0;
}

int parse_command_modifiers(exarg_T *eap, char **errormsg, cmdmod_T *cmod, int skip_only)
{
    (void)eap; (void)errormsg; (void)cmod; (void)skip_only; return OK;
}

int modifier_len(char_u *cmd)
{ (void)cmd; return 0; }

void not_exiting(void) {}

int number_method(char_u *cmd)
{ (void)cmd; return 0; }

FILE *open_exfile(char_u *fname, int forceit, char *mode)
{ (void)fname; (void)forceit; (void)mode; return NULL; }

int parse_cmd_address(exarg_T *eap, char **errormsg, int silent)
{ (void)eap; (void)errormsg; (void)silent; return FAIL; }

// Save/restore current state (very shallow no-op for linking).
int save_current_state(save_state_T *sst)
{
    (void)sst; return 1;
}

void restore_current_state(save_state_T *sst)
{
    (void)sst;
}

// ---------------------------------------------------------------------------
// Additional stubs to satisfy links against ex_docmd APIs

void do_exmode(int improved) { (void)improved; }

int do_cmdline_cmd(char_u *cmd) { (void)cmd; return OK; }

// Avoid defining do_cmdline to prevent duplicate symbol with Rust excmd crate.
/* int do_cmdline(char_u *cmdline, char_u *(*fgetline)(int, void *, int, getline_opt_T), void *cookie, int flags)
{ (void)cmdline; (void)fgetline; (void)cookie; (void)flags; return OK; } */

void handle_did_throw(void) {}

int getline_equal(char_u *(*fgetline)(int, void *, int, getline_opt_T), void *cookie, char_u *(*func)(int, void *, int, getline_opt_T))
{ (void)fgetline; (void)cookie; (void)func; return FALSE; }

void *getline_cookie(char_u *(*fgetline)(int, void *, int, getline_opt_T), void *cookie)
{ (void)fgetline; return cookie; }

char_u *getline_peek(char_u *(*fgetline)(int, void *, int, getline_opt_T), void *cookie)
{ (void)fgetline; (void)cookie; return NULL; }

char *ex_errmsg(char *msg, char_u *arg)
{ (void)arg; return msg; }

static char empty_cstr[] = "";
char *ex_range_without_command(exarg_T *eap)
{ (void)eap; return empty_cstr; }

int checkforcmd(char_u **pp, char *cmd, int len)
{ (void)pp; (void)cmd; (void)len; return 0; }

int checkforcmd_noparen(char_u **pp, char *cmd, int len)
{ (void)pp; (void)cmd; (void)len; return 0; }

int has_cmdmod(cmdmod_T *cmod, int ignore_silent)
{ (void)cmod; (void)ignore_silent; return 0; }

int cmdmod_error(int ignore_silent)
{ (void)ignore_silent; return 0; }

int cmd_exists(char_u *name)
{ (void)name; return 0; }

void f_fullcommand(typval_T *argvars, typval_T *rettv)
{ (void)argvars; rettv->v_type = VAR_NUMBER; rettv->vval.v_number = 0; }

cmdidx_T excmd_get_cmdidx(char_u *cmd, int len)
{ (void)cmd; (void)len; return (cmdidx_T)0; }

long excmd_get_argt(cmdidx_T idx)
{ (void)idx; return 0; }

char_u *find_ex_command(exarg_T *eap, int *full, int (*lookup)(char_u *, size_t, int cmd, cctx_T *), cctx_T *cctx)
{ (void)eap; (void)full; (void)lookup; (void)cctx; return NULL; }

linenr_T get_address(exarg_T *eap, char_u **ptr, cmd_addr_T addr_type, int skip, int silent, int to_other_file, int address_count)
{ (void)eap; (void)ptr; (void)addr_type; (void)skip; (void)silent; (void)to_other_file; (void)address_count; return 0; }

void ex_ni(exarg_T *eap) { (void)eap; }

int expand_filename(exarg_T *eap, char_u **cmdlinep, char **errormsgp)
{ (void)eap; (void)cmdlinep; (void)errormsgp; return 0; }

int get_bad_opt(char_u *p, exarg_T *eap)
{ (void)p; (void)eap; return 0; }

int expand_argopt(char_u *pat, expand_T *xp, regmatch_T *rmp, char_u ***matches, int *numMatches)
{ (void)pat; (void)xp; (void)rmp; if (matches) *matches = NULL; if (numMatches) *numMatches = 0; return 0; }

int ends_excmd(int c)
{ return (c == '|' || c == NUL); }

int ends_excmd2(char_u *cmd_start, char_u *cmd)
{ (void)cmd_start; (void)cmd; return 0; }

char_u *find_nextcmd(char_u *p)
{ while (*p && *p != '|') ++p; return p; }

char_u *get_command_name(expand_T *xp, int idx)
{ (void)xp; (void)idx; return (char_u *)empty_cstr; }

int before_quit_autocmds(win_T *wp, int quit_all, int forceit)
{ (void)wp; (void)quit_all; (void)forceit; return OK; }

void ex_quit(exarg_T *eap) { (void)eap; }

void tabpage_close(int forceit) { (void)forceit; }

void tabpage_close_other(tabpage_T *tp, int forceit) { (void)tp; (void)forceit; }

void ex_stop(exarg_T *eap) { (void)eap; }

void handle_drop(int filec, char_u **filev, int split, void (*callback)(void *), void *cookie)
{ (void)filec; (void)filev; (void)split; (void)callback; (void)cookie; }

void handle_any_postponed_drop(void) {}

int expand_findfunc(char_u *pat, char_u ***files, int *numMatches)
{ (void)pat; if (files) *files = NULL; if (numMatches) *numMatches = 0; return 0; }

char *did_set_findfunc(optset_T *args)
{ (void)args; return NULL; }

void free_findfunc_option(void) {}

void ex_splitview(exarg_T *eap) { (void)eap; }

void do_exedit(exarg_T *eap, win_T *old_curwin) { (void)eap; (void)old_curwin; }

void free_cd_dir(void) {}

void post_chdir(cdscope_T scope) { (void)scope; }

void ex_cd(exarg_T *eap) { (void)eap; }

void do_sleep(long msec, int hide_cursor) { (void)msec; (void)hide_cursor; }

void ex_may_print(exarg_T *eap) { (void)eap; }

void ex_redraw(exarg_T *eap) { (void)eap; set_must_redraw(UPD_CLEAR); }

void ex_normal(exarg_T *eap) { (void)eap; }

void exec_normal_cmd(char_u *cmd, int remap, int silent) { (void)cmd; (void)remap; (void)silent; }

void exec_normal(int was_typed, int use_vpeekc, int may_use_terminal_loop)
{ (void)was_typed; (void)use_vpeekc; (void)may_use_terminal_loop; }

int find_cmdline_var(char_u *src, size_t *usedlen)
{ (void)src; if (usedlen) *usedlen = 0; return FALSE; }

char_u *eval_vars(char_u *src, char_u *srcstart, size_t *usedlen, linenr_T *lnump, char **errormsg, int *escaped, int empty_is_error)
{ (void)srcstart; (void)usedlen; (void)lnump; (void)errormsg; (void)escaped; (void)empty_is_error; return src; }

char_u *expand_sfile(char_u *arg) { return arg; }

void dialog_msg(char_u *buff, char *format, char_u *fname)
{ (void)format; (void)fname; if (buff) *buff = NUL; }

int is_loclist_cmd(int cmdidx) { (void)cmdidx; return 0; }
