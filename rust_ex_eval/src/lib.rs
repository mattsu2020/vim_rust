use std::os::raw::{c_char, c_int, c_long, c_uchar, c_void};

#[no_mangle]
pub extern "C" fn aborting() -> c_int { 0 }
#[no_mangle]
pub extern "C" fn update_force_abort() {}
#[no_mangle]
pub extern "C" fn should_abort(_retcode: c_int) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn aborted_in_try() -> c_int { 0 }
#[no_mangle]
pub extern "C" fn cause_errthrow(_mesg: *mut c_uchar, _severe: c_int, _ignore: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn free_global_msglist() {}
#[no_mangle]
pub extern "C" fn do_errthrow(_cstack: *mut c_void, _cmdname: *mut c_uchar) {}
#[no_mangle]
pub extern "C" fn do_intthrow(_cstack: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn get_exception_string(_value: *mut c_void, _type: c_int, _cmdname: *mut c_uchar, _should_free: *mut c_void) -> *mut c_char { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn throw_exception(_value: *mut c_void, _type: c_int, _cmdname: *mut c_uchar) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn discard_current_exception() {}
#[no_mangle]
pub extern "C" fn catch_exception(_excp: *mut c_void) {}
#[no_mangle]
pub extern "C" fn finish_exception(_excp: *mut c_void) {}
#[no_mangle]
pub extern "C" fn exception_state_save(_estate: *mut c_void) {}
#[no_mangle]
pub extern "C" fn exception_state_restore(_estate: *mut c_void) {}
#[no_mangle]
pub extern "C" fn exception_state_clear() {}
#[no_mangle]
pub extern "C" fn report_make_pending(_pending: c_int, _value: *mut c_void) {}
#[no_mangle]
pub extern "C" fn cmd_is_name_only(_arg: *mut c_uchar) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn ex_eval(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_if(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_endif(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_else(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_while(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_continue(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_break(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_endwhile(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_block(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_endblock(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn inside_block(_eap: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn ex_throw(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn do_throw(_cstack: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_try(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_catch(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_finally(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_endtry(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn enter_cleanup(_csp: *mut c_void) {}
#[no_mangle]
pub extern "C" fn leave_cleanup(_csp: *mut c_void) {}
#[no_mangle]
pub extern "C" fn cleanup_conditionals(_cstack: *mut c_void, _searched_cond: c_int, _inclusive: c_int) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn rewind_conditionals(_cstack: *mut c_void, _idx: c_int, _cond_type: c_int, _cond_level: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_endfunction(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn has_loop_cmd(_p: *mut c_uchar) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn parse_pattern_and_range(_incsearch_start: *mut c_void, _search_delim: *mut c_void, _skiplen: *mut c_void, _patlen: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn cmdline_init() {}
#[no_mangle]
pub extern "C" fn getcmdline(_firstc: c_int, _count: c_long, _indent: c_int, _do_concat: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn getcmdline_prompt(_firstc: c_int, _prompt: *mut c_uchar, _attr: c_int, _xp_context: c_int, _xp_arg: *mut c_uchar) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn check_opt_wim() -> c_int { 0 }
#[no_mangle]
pub extern "C" fn text_locked() -> c_int { 0 }
#[no_mangle]
pub extern "C" fn text_locked_msg() {}
#[no_mangle]
pub extern "C" fn get_text_locked_msg() -> *mut c_char { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn text_or_buf_locked() -> c_int { 0 }
#[no_mangle]
pub extern "C" fn curbuf_locked() -> c_int { 0 }
#[no_mangle]
pub extern "C" fn allbuf_locked() -> c_int { 0 }
#[no_mangle]
pub extern "C" fn getexline(_c: c_int, _cookie: *mut c_void, _indent: c_int, _options: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn getexmodeline(_promptc: c_int, _cookie: *mut c_void, _indent: c_int, _options: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn cmdline_overstrike() -> c_int { 0 }
#[no_mangle]
pub extern "C" fn cmdline_at_end() -> c_int { 0 }
#[no_mangle]
pub extern "C" fn cmdline_getvcol_cursor() -> c_int { 0 }
#[no_mangle]
pub extern "C" fn realloc_cmdbuff(_len: c_int) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn free_arshape_buf() {}
#[no_mangle]
pub extern "C" fn putcmdline(_c: c_int, _shift: c_int) {}
#[no_mangle]
pub extern "C" fn unputcmdline() {}
#[no_mangle]
pub extern "C" fn put_on_cmdline(_str: *mut c_uchar, _len: c_int, _redraw: c_int) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn cmdline_paste_str(_s: *mut c_uchar, _literally: c_int) {}
#[no_mangle]
pub extern "C" fn redrawcmdline() {}
#[no_mangle]
pub extern "C" fn redrawcmdline_ex(_do_compute_cmdrow: c_int) {}
#[no_mangle]
pub extern "C" fn redrawcmd() {}
#[no_mangle]
pub extern "C" fn compute_cmdrow() {}
#[no_mangle]
pub extern "C" fn cursorcmd() {}
#[no_mangle]
pub extern "C" fn gotocmdline(_clr: c_int) {}
#[no_mangle]
pub extern "C" fn vim_strsave_fnameescape(_fname: *mut c_uchar, _what: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn escape_fname(_pp: *mut *mut c_uchar) {}
#[no_mangle]
pub extern "C" fn tilde_replace(_orig_pat: *mut c_uchar, _num_files: c_int, _files: *mut *mut c_uchar) {}
#[no_mangle]
pub extern "C" fn get_cmdline_info() -> *mut c_void { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn f_getcmdcomplpat(_argvars: *mut c_void, _rettv: *mut c_void) {}
#[no_mangle]
pub extern "C" fn f_getcmdcompltype(_argvars: *mut c_void, _rettv: *mut c_void) {}
#[no_mangle]
pub extern "C" fn f_getcmdline(_argvars: *mut c_void, _rettv: *mut c_void) {}
#[no_mangle]
pub extern "C" fn f_getcmdpos(_argvars: *mut c_void, _rettv: *mut c_void) {}
#[no_mangle]
pub extern "C" fn f_getcmdprompt(_argvars: *mut c_void, _rettv: *mut c_void) {}
#[no_mangle]
pub extern "C" fn f_getcmdscreenpos(_argvars: *mut c_void, _rettv: *mut c_void) {}
#[no_mangle]
pub extern "C" fn f_getcmdtype(_argvars: *mut c_void, _rettv: *mut c_void) {}
#[no_mangle]
pub extern "C" fn f_setcmdline(_argvars: *mut c_void, _rettv: *mut c_void) {}
#[no_mangle]
pub extern "C" fn f_setcmdpos(_argvars: *mut c_void, _rettv: *mut c_void) {}
#[no_mangle]
pub extern "C" fn get_cmdline_firstc() -> c_int { 0 }
#[no_mangle]
pub extern "C" fn get_list_range(_str: *mut *mut c_uchar, _num1: *mut c_void, _num2: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn did_set_cedit(_args: *mut c_void) -> *mut c_char { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn is_in_cmdwin() -> c_int { 0 }
#[no_mangle]
pub extern "C" fn script_get(_eap: *mut c_void, _cmd: *mut c_uchar) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn get_user_input(_argvars: *mut c_void, _rettv: *mut c_void, _inputdialog: c_int, _secret: c_int) {}
#[no_mangle]
pub extern "C" fn f_wildtrigger(_argvars: *mut c_void, _rettv: *mut c_void) {}
