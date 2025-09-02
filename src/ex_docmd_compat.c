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
