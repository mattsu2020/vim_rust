// Minimal compatibility layer that bridges Vim's channel API to the Rust
// implementation provided in src/rust/channel/.
#if defined(FEAT_JOB_CHANNEL) || defined(PROTO)
#include "vim.h"
#include "channel_rs.h"
#include <stdbool.h>
// FFI declarations are provided by channel_rs.h, which exposes the Rust
// channel implementation under rs_* symbols.

typedef struct rs_map_S {
    channel_T *ch;
    void      *rs; // opaque Rust Channel*
} rs_map_T;

static rs_map_T *rs_maps = NULL;
static size_t rs_maps_len = 0;

static void rs_map_set(channel_T *ch, void *rs)
{
    for (size_t i = 0; i < rs_maps_len; ++i)
        if (rs_maps[i].ch == ch) { rs_maps[i].rs = rs; return; }
    rs_maps = (rs_map_T *)vim_realloc(rs_maps, sizeof(rs_map_T) * (rs_maps_len + 1));
    rs_maps[rs_maps_len].ch = ch;
    rs_maps[rs_maps_len].rs = rs;
    rs_maps_len++;
}

static void *rs_map_get(channel_T *ch)
{
    for (size_t i = 0; i < rs_maps_len; ++i)
        if (rs_maps[i].ch == ch) return rs_maps[i].rs;
    return NULL;
}

static void rs_map_del(channel_T *ch)
{
    for (size_t i = 0; i < rs_maps_len; ++i)
        if (rs_maps[i].ch == ch) {
            if (i + 1 < rs_maps_len)
                rs_maps[i] = rs_maps[rs_maps_len - 1];
            --rs_maps_len;
            if (rs_maps_len == 0) {
                vim_free(rs_maps);
                rs_maps = NULL;
            }
            return;
        }
}

// Create a new empty channel structure.
channel_T *add_channel(void)
{
    channel_T *ch = ALLOC_CLEAR_ONE(channel_T);
    if (ch != NULL)
        ch->ch_refcount = 1;
    return ch;
}

// Open a TCP channel via Rust implementation. Ignores waittime and nb_close_cb.
channel_T *channel_open(const char *hostname, int port, int waittime, void (*nb_close_cb)(void))
{
    (void)waittime; (void)nb_close_cb;
    size_t len = STRLEN(hostname) + 16;
    char *addr = alloc(len);
    if (addr == NULL)
        return NULL;
    vim_snprintf(addr, len, "%s:%d", hostname, port);
    Channel *rs = rs_channel_open(addr);
    vim_free(addr);
    if (rs == NULL)
        return NULL;
    channel_T *ch = add_channel();
    if (ch == NULL) {
        rs_channel_close(rs);
        return NULL;
    }
    rs_map_set(ch, rs);
    return ch;
}

// Send data through the channel. "part" is ignored for the Rust transport.
int channel_send(channel_T *channel, ch_part_T part, char_u *buf_arg, int len_arg, char *fun)
{
    (void)part; (void)fun;
    Channel *rs = (Channel *)rs_map_get(channel);
    if (rs == NULL)
        return FAIL;
    if (len_arg < 0)
        len_arg = (int)STRLEN(buf_arg);
    int rc = rs_channel_send(rs, (const char *)buf_arg, (size_t)len_arg);
    return rc == 0 ? OK : FAIL;
}


// Close the channel and free resources.
void channel_close(channel_T *channel, int invoke_close_cb)
{
    (void)invoke_close_cb;
    Channel *rs = (Channel *)rs_map_get(channel);
    if (rs != NULL)
        rs_channel_close(rs);
    rs_map_del(channel);
}

// Clear and free the channel.  This mimics legacy behavior.
void channel_clear(channel_T *channel)
{
    if (channel == NULL)
        return;
    channel_close(channel, TRUE);
    vim_free(channel);
}

// Stubs and simple defaults to satisfy callers.
int channel_can_write_to(channel_T *channel) { return rs_map_get(channel) != NULL; }
int channel_is_open(channel_T *channel) { return rs_map_get(channel) != NULL; }
void channel_write_in(channel_T *channel) { (void)channel; }
void ch_close_part(channel_T *channel, ch_part_T part) { (void)channel; (void)part; }
void channel_set_pipes(channel_T *channel, sock_T in, sock_T out, sock_T err) { (void)channel; (void)in; (void)out; (void)err; }
void channel_set_job(channel_T *channel, job_T *job, jobopt_T *options) { (void)channel; (void)job; (void)options; }
readq_T *channel_peek(channel_T *channel, ch_part_T part) { (void)channel; (void)part; return NULL; }
char_u *channel_get(channel_T *channel, ch_part_T part, int *outlen) { (void)channel; (void)part; if (outlen) *outlen = 0; return NULL; }
void channel_consume(channel_T *channel, ch_part_T part, int len) { (void)channel; (void)part; (void)len; }
int channel_collapse(channel_T *channel, ch_part_T part, int want_nl) { (void)channel; (void)part; (void)want_nl; return OK; }
void channel_set_nonblock(channel_T *channel, ch_part_T part) { (void)channel; (void)part; }

// No-op implementations for unused APIs in this build path
int has_any_channel(void) { return rs_maps_len > 0; }
int channel_still_useful(channel_T *channel) { (void)channel; return TRUE; }
int channel_can_close(channel_T *channel) { (void)channel; return TRUE; }
int channel_unref(channel_T *channel) { (void)channel; return 0; }
int free_unused_channels_contents(int copyID, int mask) { (void)copyID; (void)mask; return 0; }
void free_unused_channels(int copyID, int mask) { (void)copyID; (void)mask; }
void channel_gui_register_all(void) {}
void channel_buffer_free(buf_T *buf) { (void)buf; }
void channel_write_any_lines(void) {}
void channel_write_new_lines(buf_T *buf) { (void)buf; }
char_u *channel_first_nl(readq_T *node) { (void)node; return NULL; }
void channel_free_all(void) {}
int channel_in_blocking_wait(void) { return FALSE; }
channel_T *get_channel_arg(typval_T *tv, int check_open, int reading, ch_part_T part) { (void)tv; (void)check_open; (void)reading; (void)part; return NULL; }
void channel_handle_events(int only_keep_open) { (void)only_keep_open; }
int channel_any_keep_open(void) { return FALSE; }
int channel_poll_setup(int nfd_in, void *fds_in, int *towait) { (void)nfd_in; (void)fds_in; (void)towait; return 0; }
int channel_poll_check(int ret_in, void *fds_in) { (void)ret_in; (void)fds_in; return 0; }
int channel_select_setup(int maxfd_in, void *rfds_in, void *wfds_in, struct timeval *tv, struct timeval **tvp) { (void)maxfd_in; (void)rfds_in; (void)wfds_in; (void)tv; (void)tvp; return 0; }
int channel_select_check(int ret_in, void *rfds_in, void *wfds_in) { (void)ret_in; (void)rfds_in; (void)wfds_in; return 0; }
int channel_parse_messages(void) { return 0; }
int channel_any_readahead(void) { return 0; }
int set_ref_in_channel(int copyID) { (void)copyID; return 0; }

// Builtin Vimscript function stubs for channel_* to satisfy linkage
void f_ch_canread(typval_T *argvars, typval_T *rettv) { (void)argvars; rettv->v_type = VAR_NUMBER; rettv->vval.v_number = 0; }
void f_ch_close(typval_T *argvars, typval_T *rettv) { (void)argvars; (void)rettv; }
void f_ch_close_in(typval_T *argvars, typval_T *rettv) { (void)argvars; (void)rettv; }
void f_ch_evalexpr(typval_T *argvars, typval_T *rettv) { (void)argvars; rettv->v_type = VAR_SPECIAL; rettv->vval.v_number = VVAL_NULL; }
void f_ch_evalraw(typval_T *argvars, typval_T *rettv) { (void)argvars; rettv->v_type = VAR_SPECIAL; rettv->vval.v_number = VVAL_NULL; }
void f_ch_getbufnr(typval_T *argvars, typval_T *rettv) { (void)argvars; rettv->v_type = VAR_NUMBER; rettv->vval.v_number = -1; }
void f_ch_getjob(typval_T *argvars, typval_T *rettv) { (void)argvars; rettv->v_type = VAR_NUMBER; rettv->vval.v_number = 0; }
void f_ch_info(typval_T *argvars, typval_T *rettv) { (void)argvars; rettv->v_type = VAR_DICT; rettv->vval.v_dict = NULL; }
void f_ch_open(typval_T *argvars, typval_T *rettv) { (void)argvars; rettv->v_type = VAR_NUMBER; rettv->vval.v_number = 0; }
void f_ch_read(typval_T *argvars, typval_T *rettv) { (void)argvars; rettv->v_type = VAR_STRING; rettv->vval.v_string = (char_u *)vim_strsave((char_u *)""); }
void f_ch_readblob(typval_T *argvars, typval_T *rettv) { (void)argvars; rettv->v_type = VAR_BLOB; rettv->vval.v_blob = NULL; }
void f_ch_readraw(typval_T *argvars, typval_T *rettv) { (void)argvars; rettv->v_type = VAR_STRING; rettv->vval.v_string = (char_u *)vim_strsave((char_u *)""); }
void f_ch_sendexpr(typval_T *argvars, typval_T *rettv) { (void)argvars; rettv->v_type = VAR_NUMBER; rettv->vval.v_number = 0; }
void f_ch_sendraw(typval_T *argvars, typval_T *rettv) { (void)argvars; rettv->v_type = VAR_NUMBER; rettv->vval.v_number = 0; }
void f_ch_setoptions(typval_T *argvars, typval_T *rettv) { (void)argvars; rettv->v_type = VAR_NUMBER; rettv->vval.v_number = 0; }
void f_ch_status(typval_T *argvars, typval_T *rettv) { (void)argvars; rettv->v_type = VAR_STRING; rettv->vval.v_string = (char_u *)vim_strsave((char_u *)"closed"); }
char_u *channel_to_string_buf(typval_T *varp, char_u *buf) { (void)varp; buf[0] = NUL; return buf; }

// Provide a trivial fallback for eval_expr_rs so that linking succeeds
bool eval_expr_rs(const char *expr, typval_T *result)
{
    (void)expr;
    if (result)
    {
        result->v_type = VAR_NUMBER;
        result->vval.v_number = 0;
    }
    return false;
}

#endif // FEAT_JOB_CHANNEL || PROTO
