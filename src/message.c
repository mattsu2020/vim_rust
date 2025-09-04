#include "vim.h"
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

// FFI bridge to Rust message handling
extern void rs_queue_message(char *msg, int level);

static void
send_formatted(int level, const char *fmt, va_list ap)
{
    char buf[2048];
    vsnprintf(buf, sizeof(buf), fmt, ap);
    rs_queue_message(buf, level);
}

int
smsg(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    send_formatted(0, fmt, ap);
    va_end(ap);
    return 0;
}

int
semsg(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    send_formatted(2, fmt, ap);
    va_end(ap);
    return 0;
}

void
siemsg(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    send_formatted(2, fmt, ap);
    va_end(ap);
}

void
emsg(const char_u *msg)
{
    if (msg != NULL)
        rs_queue_message((char *)msg, 2);
}

void
msg_putchar(int c)
{
    char buf[2] = { (char)c, '\0' };
    rs_queue_message(buf, 0);
}

void
msg_puts(const char *s)
{
    if (s != NULL)
        rs_queue_message((char *)s, 0);
}

void
msg_puts_attr(char *s, int attr)
{
    (void)attr;
    msg_puts(s);
}

void
msg_puts_title(const char *s)
{
    if (s != NULL)
        rs_queue_message((char *)s, 0);
}

void
msg_outtrans(char_u *s)
{
    if (s != NULL)
        rs_queue_message((char *)s, 0);
}

int
msg_outtrans_attr(char_u *s, int attr)
{
    (void)attr;
    if (s == NULL)
        return 0;
    rs_queue_message((char *)s, 0);
    return (int)strlen((const char *)s);
}

char *
msg_outtrans_long_attr(char *s, int attr)
{
    (void)attr;
    if (s != NULL)
        rs_queue_message(s, 0);
    return s;
}

void
give_warning(char_u *msg, int hl)
{
    (void)hl;
    if (msg != NULL)
        rs_queue_message((char *)msg, 1);
}

int
msg(char_u *s)
{
    if (s != NULL)
        rs_queue_message((char *)s, 0);
    return 0;
}

void
msg_prt_line(char_u *s, int list)
{
    (void)list;
    if (s != NULL)
        rs_queue_message((char *)s, 0);
}

void
verb_msg(char *s)
{
    if (s != NULL)
        rs_queue_message(s, 0);
}

// Basic implementations for previously empty stubs -----------------------
// Clear from the current message position to the end of the screen.
void
msg_clr_eos(void)
{
    // Use a simple ANSI escape sequence to clear to the end of the screen.
    rs_queue_message("\033[J", 0);
}

// Mark the start of a message.  For this minimal implementation we only
// reset the "msg_didout" flag so that following output is displayed on a new
// line, similar to the real Vim behaviour.
void
msg_start(void)
{
    msg_didout = FALSE;
}

// Finish putting a message on the screen.  Nothing special needs to happen
// here but the function is kept for compatibility.
void
msg_end(void)
{
}

// Return non-zero when the message should be filtered out.  We simply check
// if messages are silenced globally.
int
message_filtered(char_u *s)
{
    (void)s;
    return msg_silent > 0;
}

// Enter/leave a verbose section.  We adjust "msg_silent" so that nested
// calls keep track of the current level.
void
verbose_enter(void)
{
    ++msg_silent;
}
void
verbose_leave(void)
{
    if (msg_silent > 0)
        --msg_silent;
}

// Truncate a message and return the (possibly truncated) string.  The
// truncation happens using a static buffer so the caller must use or copy the
// result before the next call.
char *
msg_trunc_attr(char *s, int use_history, int attr)
{
    static char buf[IOSIZE];

    (void)use_history;
    (void)attr;

    if (s == NULL)
        return NULL;

    trunc_string(s, buf, IOSIZE - 1, IOSIZE);
    msg_hist_off = FALSE;
    return buf;
}
void
set_keep_msg(char_u *p, int attr)
{
    (void)attr;
    keep_msg = p;
}
void
trunc_string(const char *src, char *dst, int maxlen, int dstlen)
{
    (void)maxlen;
    if (src && dst && dstlen > 0)
        snprintf(dst, (size_t)dstlen, "%s", src);
}
void
verbose_enter_scroll(void)
{
    verbose_enter();
}
void
verbose_leave_scroll(void)
{
    verbose_leave();
}
void
msg_warn_missing_clipboard(void)
{
    rs_queue_message("missing clipboard support", 1);
}
void
msg_outnum(long n)
{
    char buf[64];
    snprintf(buf, sizeof(buf), "%ld", n);
    rs_queue_message(buf, 0);
}
void
emsg_invreg(int c)
{
    char buf[64];
    snprintf(buf, sizeof(buf), "Invalid register: %c", c);
    rs_queue_message(buf, 2);
}
void
internal_error(const char *where)
{
    if (where)
        rs_queue_message((char *)where, 2);
}
void
internal_error_no_abort(const char *where)
{
    internal_error(where);
}
void
msg_putchar_attr(int c, int attr)
{
    (void)attr;
    msg_putchar(c);
}
void
iemsg(const char *s)
{
    if (s)
        rs_queue_message((char *)s, 2);
}
void
wait_return(int redraw)
{
    (void)redraw;
    did_wait_return = TRUE;
}
int
vim_dialog_yesno(int type, char_u *title, char_u *message, int buttons)
{
    (void)type;
    (void)title;
    (void)buttons;
    if (message)
        rs_queue_message((char *)message, 0);
    return VIM_YES;
}
void
ch_logfile(char_u *fname, char_u *mode)
{
    (void)fname;
    (void)mode;
}
void
may_clear_sb_text(void)
{
}
int
do_dialog(int type,
          char_u *title,
          char_u *message,
          char_u *buttons,
          int dfltbutton,
          char_u *textfield,
          int ex_cmd)
{
    (void)type;
    (void)title;
    (void)buttons;
    (void)textfield;
    (void)ex_cmd;
    if (message)
        rs_queue_message((char *)message, 0);
    return dfltbutton <= 0 ? 1 : dfltbutton;
}
char_u *
str2special_save(char_u *src, int do_special, int keep_screen_char)
{
    (void)do_special;
    (void)keep_screen_char;
    if (!src)
        return NULL;
    size_t len = strlen((const char *)src);
    char_u *p = (char_u *)malloc(len + 1);
    if (p)
    {
        memcpy(p, src, len + 1);
    }
    return p;
}
void
msg_source(int attr)
{
    (void)attr;
}
void
windgoto(int row, int col)
{
    (void)row;
    (void)col;
}
