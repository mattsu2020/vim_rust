#include "vim.h"
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

// Bridge to Rust message queue if available.
extern void rs_queue_message(char *msg, int level);

static void vprint_prefixed(const char *prefix, const char *fmt, va_list ap)
{
    if (prefix != NULL) fputs(prefix, stderr);
    vfprintf(stderr, fmt, ap);
    fputc('\n', stderr);
}

static void bridge_info(const char *s)
{
    if (s) rs_queue_message((char *)s, 0);
}

static void bridge_warn(const char *s)
{
    if (s) rs_queue_message((char *)s, 1);
}

static void bridge_error(const char *s)
{
    if (s) rs_queue_message((char *)s, 2);
}

int smsg(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    char buf[2048];
    vsnprintf(buf, sizeof(buf), fmt, ap);
    fputs(buf, stderr);
    fputc('\n', stderr);
    bridge_info(buf);
    va_end(ap);
    return 0;
}

int semsg(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    char buf[2048];
    vsnprintf(buf, sizeof(buf), fmt, ap);
    fputs("E: ", stderr);
    fputs(buf, stderr);
    fputc('\n', stderr);
    bridge_error(buf);
    va_end(ap);
    return 0;
}

void siemsg(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    char buf[2048];
    vsnprintf(buf, sizeof(buf), fmt, ap);
    fputs("E: ", stderr);
    fputs(buf, stderr);
    fputc('\n', stderr);
    bridge_error(buf);
    va_end(ap);
}

void emsg(const char_u *msg)
{
    if (msg == NULL) return;
    fprintf(stderr, "E: %s\n", (const char *)msg);
    bridge_error((const char *)msg);
}

void msg_putchar(int c)
{
    fputc(c, stderr);
}

void msg_puts(const char *s)
{
    if (s != NULL) {
        fputs(s, stderr);
        bridge_info(s);
    }
}

void msg_puts_attr(char *s, int attr)
{
    (void)attr;
    msg_puts(s);
}

void msg_puts_title(const char *s)
{
    if (s != NULL) {
        fprintf(stderr, "%s\n", s);
        bridge_info(s);
    }
}

void msg_outtrans(char_u *s)
{
    if (s != NULL) {
        fputs((const char *)s, stderr);
        bridge_info((const char *)s);
    }
}

void msg_clr_eos(void) {}
void msg_start(void) {}
void msg_end(void) {}

int message_filtered(char_u *s)
{
    (void)s;
    return 0;
}

void verbose_enter(void) {}
void verbose_leave(void) {}

void give_warning(char_u *msg, int hl)
{
    (void)hl;
    if (msg != NULL) {
        fprintf(stderr, "W: %s\n", (const char *)msg);
        bridge_warn((const char *)msg);
    }
}

int msg(char_u *s)
{
    if (s != NULL) {
        fprintf(stderr, "%s\n", (const char *)s);
        bridge_info((const char *)s);
    }
    return 0;
}

char *msg_trunc_attr(char *s, int use_history, int attr)
{
    (void)use_history;
    (void)attr;
    // Reset msg_hist_off as callers expect
    msg_hist_off = FALSE;
    return s;
}

void set_keep_msg(char_u *p, int attr)
{
    (void)attr;
    keep_msg = p;
}

void trunc_string(const char *src, char *dst, int maxlen, int dstlen)
{
    (void)maxlen;
    if (src == NULL || dst == NULL || dstlen <= 0) return;
    // Simple copy, ensure NUL-terminated
    snprintf(dst, (size_t)dstlen, "%s", src);
}

void verbose_enter_scroll(void) {}
void verbose_leave_scroll(void) {}

void msg_warn_missing_clipboard(void)
{
    fprintf(stderr, "W: missing clipboard support\n");
}

int msg_outtrans_attr(char_u *s, int attr)
{
    (void)attr;
    if (s == NULL) return 0;
    fputs((const char *)s, stderr);
    return (int)strlen((const char *)s);
}

char *msg_outtrans_long_attr(char *s, int attr)
{
    (void)attr;
    if (s != NULL) fputs(s, stderr);
    return s;
}

void msg_advance(int col)
{
    (void)col;
}

void msg_starthere(void) {}

void internal_error(const char *where)
{
    if (where) fprintf(stderr, "Internal error: %s\n", where);
}

int msg_outtrans_len_attr(char_u *s, int len, int attr)
{
    (void)attr;
    if (s == NULL || len <= 0) return 0;
    fwrite((const char *)s, 1, (size_t)len, stderr);
    return len;
}

void msg_attr(char_u *s, int attr)
{
    (void)attr;
    if (s) fputs((const char *)s, stderr);
}

void msg_outnum(long n)
{
    fprintf(stderr, "%ld", n);
}

void emsg_invreg(int c)
{
    char buf[64];
    snprintf(buf, sizeof(buf), "Invalid register: %c", c);
    emsg((char_u *)buf);
}

void internal_error_no_abort(const char *where)
{
    internal_error(where);
}

void msg_sb_eol(void) {}

void msg_putchar_attr(int c, int attr)
{
    (void)attr;
    fputc(c, stderr);
}

void iemsg(const char *s)
{
    if (s != NULL)
    {
        fputs("I: ", stderr);
        fputs(s, stderr);
        fputc('\n', stderr);
        bridge_error(s);
    }
}

void wait_return(int redraw)
{
    (void)redraw;
    // Minimal build: do not block, just note that we would have waited
    // and mark that we used wait_return.
    did_wait_return = TRUE;
}

int vim_dialog_yesno(int type, char_u *title, char_u *message, int buttons)
{
    (void)type; (void)title; (void)buttons;
    if (message != NULL)
    {
        fputs("? ", stderr);
        fputs((const char *)message, stderr);
        fputc('\n', stderr);
    }
    // Default to YES to keep flows moving in minimal build.
    return VIM_YES;
}

void msg_prt_line(char_u *s, int list)
{
    (void)list;
    if (s != NULL)
    {
        fputs((const char *)s, stderr);
        fputc('\n', stderr);
        bridge_info((const char *)s);
    }
}

void ch_logfile(char_u *fname, char_u *mode)
{
    (void)fname; (void)mode;
    // No-op in minimal build.
}

void may_clear_sb_text(void)
{
    // No-op in minimal build.
}

int do_dialog(int type, char_u *title, char_u *message, char_u *buttons, int dfltbutton, char_u *textfield, int ex_cmd)
{
    (void)type; (void)title; (void)buttons; (void)textfield; (void)ex_cmd;
    if (message != NULL)
    {
        fputs((const char *)message, stderr);
        fputc('\n', stderr);
    }
    return dfltbutton <= 0 ? 1 : dfltbutton;
}

void verb_msg(char *s)
{
    if (s != NULL)
    {
        fputs(s, stderr);
        fputc('\n', stderr);
    }
}

char_u *str2special_save(char_u *src, int do_special, int keep_screen_char)
{
    (void)do_special; (void)keep_screen_char;
    if (src == NULL) return NULL;
    size_t len = strlen((const char *)src);
    char_u *p = (char_u *)malloc(len + 1);
    if (p == NULL) return NULL;
    memcpy(p, src, len + 1);
    return p;
}

void msg_source(int attr)
{
    (void)attr;
}

void windgoto(int row, int col)
{
    (void)row; (void)col;
}
