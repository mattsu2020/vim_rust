/* vi:set ts=8 sts=4 sw=4 noet:
 *
 * VIM - Vi IMproved	by Bram Moolenaar
 *
 * Do ":help uganda"  in Vim to read copying and usage conditions.
 * Do ":help credits" in Vim to see a list of people who contributed.
 */

/*
 * proto.h: include the (automatically generated) function prototypes
 */

/*
 * Don't include these while generating prototypes.  Prevents problems when
 * files are missing.
 */
#if !defined(PROTO) && !defined(NOPROTO)

/*
 * Machine-dependent routines.
 */
// avoid errors in function prototypes
# if !defined(FEAT_X11) && !defined(FEAT_GUI_GTK)
#  define Display int
#  define Widget int
# endif
# ifndef FEAT_GUI_GTK
#  define GdkEvent int
#  define GdkEventKey int
# endif
# ifndef FEAT_X11
#  define XImage int
# endif

# if defined(UNIX) || defined(VMS)
#  if defined(__has_include)
#   if __has_include("os_unix.pro")
#    include "os_unix.pro"
#   endif
#  endif
# endif
# ifdef MSWIN
#  include "os_win32.pro"
#  include "os_mswin.pro"
#  include "winclip.pro"
#  if (defined(__GNUC__) && !defined(__MINGW32__))
extern int _stricoll(char *a, char *b);
#  endif
# endif
# ifdef VMS
#  include "os_vms.pro"
# endif

// xdiff の mmfile_t 定義が必要なプロトタイプに先行させる
# include "xdiff/xdiff.h"

# ifdef FEAT_CRYPT
#  include "crypt.pro"
# endif

// Rust bridge headers (included when available). If present for change.c,
// prefer the Rust header over the generated .pro to reduce dependency on it.
# if defined(__has_include)
#  if __has_include("../rust_excmds/include/rust_excmds.h")
#   include "../rust_excmds/include/rust_excmds.h"
#   define HAVE_RUST_EXCMDS_HDR 1
#  endif
#  if __has_include("../rust_clientserver/include/rust_clientserver.h")
#   include "../rust_clientserver/include/rust_clientserver.h"
#   define HAVE_RUST_CLIENTSERVER_HDR 1
#  endif
#  if __has_include("../rust_eval/include/rust_eval.h")
#   include "../rust_eval/include/rust_eval.h"
#   define HAVE_RUST_EVAL_HDR 1
#  endif
#  if __has_include("../rust_change/include/rust_change.h")
#   include "../rust_change/include/rust_change.h"
#   define HAVE_RUST_CHANGE_HDR 1
#  endif
#  if __has_include("../rust_quickfix/include/rust_quickfix.h")
#   include "../rust_quickfix/include/rust_quickfix.h"
#  endif
# endif
# include "alloc.pro"
# include "autocmd.pro"
# if defined(__has_include)
#  if __has_include("buffer.pro")
#   include "buffer.pro"
#  endif
# endif
# include "bufwrite.pro"
# ifndef HAVE_RUST_CHANGE_HDR
#  include "change.pro"
# endif
# include "charset.pro"
# include "cindent.pro"
# include "clientserver.pro"
# include "cmdexpand.pro"
# include "cmdhist.pro"
# include "if_cscope.pro"
# include "debugger.pro"
# include "dict.pro"
# include "diff.pro"
# include "digraph.pro"
# include "drawline.pro"
# include "drawscreen.pro"
# include "edit.pro"
# ifdef HAVE_RUST_EVAL_HDR
// Provide eval_to_string() via Rust when eval.pro is unavailable
# else
#  if defined(__has_include)
#   if __has_include("eval.pro")
#    include "eval.pro"
#   endif
#  endif
# endif
# include "evalbuffer.pro"
# include "evalvars.pro"
# include "evalwindow.pro"
# ifndef HAVE_RUST_EXCMDS_HDR
#  include "ex_cmds.pro"
#  include "ex_cmds2.pro"
#  include "ex_docmd.pro"
# endif
# ifdef HAVE_RUST_EXCMDS_HDR
// rust_excmds を使う場合、ex_docmd.pro の一部シンボルだけを手動で宣言する
int ends_excmd(int c);
char_u *expand_sfile(char_u *arg);
char_u *may_get_cmd_block(void *eap, char_u *cmd, char_u **tofree);
int should_abort(int reset);
void do_modelines(int flags);

// 生成されない .pro に依存するシンボルの最小スタブ（ビルド通過用）
// 実際の動作は他の Rust 実装で置き換える想定。
#  ifndef last_set_msg
#   define last_set_msg(x) ((void)0)
#  endif
#  ifndef set_string_option_direct
#   define set_string_option_direct(name, idx, val, flags, sid) ((void)0)
#  endif
#  ifndef set_bufref
#   define set_bufref(refp, buf) ((void)0)
#  endif
#  ifndef bufref_valid
#   define bufref_valid(refp) (1)
#  endif
#  ifndef bt_prompt
#   define bt_prompt(buf) (0)
#  endif
#  ifndef aborting
#   define aborting() (0)
#  endif
#  ifndef buflist_findnr
#   define buflist_findnr(nr) ((void*)0)
#  endif
// commonly missing helpers
#  ifndef check_restricted
#   define check_restricted() (0)
#  endif
#  ifndef mch_dirname
#   define mch_dirname(buf, maxlen) (FAIL)
#  endif
/* bufwrite.c 等が参照する OS/NetBeans 依存関数の最小スタブ */
#  ifndef enc2macroman
#   define enc2macroman(...) (0)
#  endif
#  ifndef check_secure
#   define check_secure() (0)
#  endif
#  ifndef bt_nofilename
#   define bt_nofilename(buf) (0)
#  endif
#  ifndef netbeans_active
#   define netbeans_active() (0)
#  endif
#  ifndef isNetbeansBuffer
#   define isNetbeansBuffer(buf) (0)
#  endif
#  ifndef isNetbeansModified
#   define isNetbeansModified(buf) (0)
#  endif
#  ifndef netbeans_save_buffer
#   define netbeans_save_buffer(buf) ((void)0)
#  endif
#  ifndef mch_nodetype
#   define mch_nodetype(name) (0)
#  endif
#  ifndef mch_get_acl
#   define mch_get_acl(name) ((vim_acl_T)0)
#  endif
#  ifndef mch_setperm
#   define mch_setperm(name,perm) (0)
#  endif
#  ifndef mch_getperm
#   define mch_getperm(name) (0)
#  endif
#  ifndef mch_fsetperm
#   define mch_fsetperm(fd,perm) (0)
#  endif
#  ifndef mch_set_acl
#   define mch_set_acl(name,acl) ((void)0)
#  endif
#  ifndef mch_free_acl
#   define mch_free_acl(acl) ((void)0)
#  endif
#  ifndef mch_hide
#   define mch_hide(name) (0)
#  endif
#  ifndef buf_setino
#   define buf_setino(buf) ((void)0)
#  endif
# endif
# if defined(__has_include)
#  if __has_include("ex_eval.pro")
#   include "ex_eval.pro"
#  endif
# endif
# if defined(__has_include)
#  if __has_include("ex_getln.pro")
#   include "ex_getln.pro"
#  endif
# endif
# include "fileio.pro"
# include "filepath.pro"
# include "findfile.pro"
# include "float.pro"
# include "fold.pro"
# include "getchar.pro"
# if defined(__has_include)
#  if __has_include("gc.pro")
#   include "gc.pro"
#  endif
# endif
# include "gui_xim.pro"
# include "hardcopy.pro"
# include "hashtab.pro"
# include "help.pro"
# if defined(__has_include)
#  if __has_include("highlight.pro")
#   include "highlight.pro"
#  endif
# endif
# include "indent.pro"
# include "insexpand.pro"
# include "list.pro"
# include "locale.pro"
/* logfile.c をビルド対象から外しているため、対応するプロトタイプも除外 */
/* # include "logfile.pro" */
# include "map.pro"
# if defined(__has_include)
#  if __has_include("mark.pro")
#   include "mark.pro"
#  endif
# endif
# if defined(__has_include)
#  if __has_include("match.pro")
#   include "match.pro"
#  endif
# endif
# include "memfile.pro"
# include "memline.pro"
# ifdef FEAT_MENU
#  include "menu.pro"
# endif
# ifdef FEAT_ARABIC
# endif
# ifdef FEAT_VIMINFO
#  include "viminfo.pro"
# endif

// These prototypes cannot be produced automatically.
int smsg(const char *, ...) ATTRIBUTE_COLD ATTRIBUTE_FORMAT_PRINTF(1, 2);

int smsg_attr(int, const char *, ...) ATTRIBUTE_FORMAT_PRINTF(2, 3);

int smsg_attr_keep(int, const char *, ...) ATTRIBUTE_FORMAT_PRINTF(2, 3);

// These prototypes cannot be produced automatically.
int semsg(const char *, ...) ATTRIBUTE_COLD ATTRIBUTE_FORMAT_PRINTF(1, 2);

// These prototypes cannot be produced automatically.
void siemsg(const char *, ...) ATTRIBUTE_COLD ATTRIBUTE_FORMAT_PRINTF(1, 2);

int vim_snprintf_add(char *, size_t, const char *, ...) ATTRIBUTE_FORMAT_PRINTF(3, 4);

int vim_snprintf(char *, size_t, const char *, ...) ATTRIBUTE_FORMAT_PRINTF(3, 4);
size_t vim_snprintf_safelen(char *, size_t, const char *, ...) ATTRIBUTE_FORMAT_PRINTF(3, 4);

int vim_vsnprintf(char *str, size_t str_m, const char *fmt, va_list ap)
	ATTRIBUTE_FORMAT_PRINTF(3, 0);
int vim_vsnprintf_typval(char *str, size_t str_m, const char *fmt, va_list ap, typval_T *tvs)
	ATTRIBUTE_FORMAT_PRINTF(3, 0);

# include "message.pro"
# include "misc1.pro"
# include "misc2.pro"
# ifndef HAVE_STRPBRK	    // not generated automatically from misc2.c
char_u *vim_strpbrk(char_u *s, char_u *charset);
# endif
# ifndef HAVE_QSORT
// Use our own qsort(), don't define the prototype when not used.
void qsort(void *base, size_t elm_count, size_t elm_size, int (*cmp)(const void *, const void *));
# endif
# include "mouse.pro"
# include "move.pro"
# include "mbyte.pro"
# ifdef VIMDLL
// Function name differs when VIMDLL is defined
int mbyte_im_get_status(void);
void mbyte_im_set_active(int active_arg);
# endif
# include "normal.pro"
# include "ops.pro"
# include "option.pro"
# include "popupmenu.pro"
# if defined(FEAT_PROFILE) || defined(FEAT_RELTIME)
#  include "profiler.pro"
# endif
#ifdef FEAT_WAYLAND
# include "wayland.pro"
#endif
# include "register.pro"
# include "screen.pro"
# include "session.pro"
# if defined(FEAT_CRYPT) || defined(FEAT_PERSISTENT_UNDO)
#  include "sha256.pro"
# endif
# if defined(__has_include)
#  if __has_include("fuzzy.pro")
#   include "fuzzy.pro"
#  endif
# endif
# include "search.pro"
# include "sound.pro"
# include "spell.pro"
# include "strings.pro"
# include "syntax.pro"
# include "tag.pro"
# include "term.pro"
# ifdef FEAT_TERMINAL
#  if defined(__has_include)
#   if __has_include("terminal.pro")
#    include "terminal.pro"
#   endif
#  else
    // Fallback: include only when available in tree
#   include "terminal.pro"
#  endif
# endif
# ifdef FEAT_PROP_POPUP
#  include "popupwin.pro"
#  include "textprop.pro"
# endif
/* testing.c もビルド対象外のため除外 */
/* # include "testing.pro" */
# include "textobject.pro"
# include "time.pro"
# include "tuple.pro"
# include "typval.pro"
# include "ui.pro"
# include "undo.pro"
# include "userfunc.pro"
# include "version.pro"
# include "vim9script.pro"
# ifdef FEAT_EVAL
// include vim9.h here, the types defined there are used by function arguments.
#  include "vim9.h"
#  include "vim9cmds.pro"
#  include "vim9compile.pro"
#  include "vim9execute.pro"
#  include "vim9expr.pro"
#  include "vim9generics.pro"
#  include "vim9instr.pro"
#  include "vim9type.pro"
# endif
# include "window.pro"

# ifdef FEAT_LUA
#  include "if_lua.pro"
# endif

# ifdef FEAT_MZSCHEME
#  include "if_mzsch.pro"
# endif

# ifdef FEAT_PYTHON
#  include "if_python.pro"
# endif

# ifdef FEAT_PYTHON3
#  include "if_python3.pro"
# endif

# ifdef FEAT_TCL
#  include "if_tcl.pro"
# endif

# ifdef FEAT_RUBY
#  include "if_ruby.pro"
# endif

// Ugly solution for "BalloonEval" not being defined while it's used in some
// .pro files.
# ifndef FEAT_BEVAL
#  define BalloonEval int
# endif
# if defined(FEAT_BEVAL) || defined(FEAT_PROP_POPUP)
#  include "beval.pro"
# endif

# ifdef FEAT_NETBEANS_INTG
# endif
# ifdef FEAT_JOB_CHANNEL
#  include "job.pro"
#  if defined(__has_include)
#   if __has_include("channel.pro")
#    include "channel.pro"
#   endif
#  endif
# endif

# ifdef FEAT_EVAL
// Not generated automatically so that we can add an extra attribute.
void ch_log(channel_T *ch, const char *fmt, ...) ATTRIBUTE_FORMAT_PRINTF(2, 3);
void ch_error(channel_T *ch, const char *fmt, ...) ATTRIBUTE_FORMAT_PRINTF(2, 3);
# endif

# if defined(FEAT_GUI) || defined(FEAT_JOB_CHANNEL)
#  if defined(UNIX) || defined(MACOS_X) || defined(VMS)
#   include "pty.pro"
#  endif
# endif

# ifdef FEAT_GUI
#  include "gui.pro"
#  if !defined(HAVE_SETENV) && !defined(HAVE_PUTENV) && !defined(VMS)
extern int putenv(const char *string);			// in misc2.c
#   ifdef USE_VIMPTY_GETENV
extern char_u *vimpty_getenv(const char_u *string);	// in misc2.c
#   endif
#  endif
#  ifdef FEAT_GUI_MSWIN
#   include "gui_w32.pro"
#  endif
#  ifdef FEAT_GUI_GTK
#   include "gui_gtk.pro"
#   include "gui_gtk_x11.pro"
#  endif
#  ifdef FEAT_GUI_MOTIF
#   include "gui_motif.pro"
#   include "gui_xmdlg.pro"
#  endif
#  ifdef FEAT_GUI_HAIKU
#   include "gui_haiku.pro"
#  endif
#  ifdef FEAT_GUI_X11
#   include "gui_x11.pro"
#  endif
#  ifdef FEAT_GUI_PHOTON
#   include "gui_photon.pro"
#  endif
# endif	// FEAT_GUI

# ifdef FEAT_OLE
#  include "if_ole.pro"
# endif

/*
 * The perl include files pollute the namespace, therefore proto.h must be
 * included before the perl include files.  But then CV is not defined, which
 * is used in if_perl.pro.  To get around this, the perl prototype files are
 * not included here for the perl files.  Use a dummy define for CV for the
 * other files.
 */
# if defined(FEAT_PERL) && !defined(IN_PERL_FILE)
#  define CV void
#  include "if_perl.pro"
#  include "if_perlsfio.pro"
# endif

# ifdef MACOS_CONVERT
#  if defined(__has_include)
#   if __has_include("os_mac_conv.pro")
#    include "os_mac_conv.pro"
#   endif
#  endif
# endif
# ifdef MACOS_X
#  include "os_macosx.pro"
# endif
# if defined(MACOS_X_DARWIN) && defined(FEAT_CLIPBOARD) && !defined(FEAT_GUI)
// functions in os_macosx.m
void clip_mch_lose_selection(Clipboard_T *cbd);
int clip_mch_own_selection(Clipboard_T *cbd);
void clip_mch_request_selection(Clipboard_T *cbd);
void clip_mch_set_selection(Clipboard_T *cbd);
# endif
#endif // !PROTO && !NOPROTO
