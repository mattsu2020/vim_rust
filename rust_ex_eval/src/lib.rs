use libc::c_int;
#[no_mangle]
pub extern "C" fn aborting() -> c_int { 0 }

#[no_mangle]
pub extern "C" fn update_force_abort() { }

#[no_mangle]
pub extern "C" fn should_abort(_a0:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn aborted_in_try() -> c_int { 0 }

#[no_mangle]
pub extern "C" fn cause_errthrow(_a0:usize, _a1:usize, _a2:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn free_global_msglist() { }

#[no_mangle]
pub extern "C" fn do_errthrow(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn do_intthrow(_a0:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn get_exception_string(_a0:usize, _a1:usize, _a2:usize, _a3:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn throw_exception(_a0:usize, _a1:usize, _a2:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn discard_current_exception() { }

#[no_mangle]
pub extern "C" fn catch_exception(_a0:usize) { }

#[no_mangle]
pub extern "C" fn finish_exception(_a0:usize) { }

#[no_mangle]
pub extern "C" fn exception_state_save(_a0:usize) { }

#[no_mangle]
pub extern "C" fn exception_state_restore(_a0:usize) { }

#[no_mangle]
pub extern "C" fn exception_state_clear() { }

#[no_mangle]
pub extern "C" fn report_make_pending(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn cmd_is_name_only(_a0:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn ex_eval(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_if(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_endif(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_else(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_while(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_continue(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_break(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_endwhile(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_block(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_endblock(_a0:usize) { }

#[no_mangle]
pub extern "C" fn inside_block(_a0:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn ex_throw(_a0:usize) { }

#[no_mangle]
pub extern "C" fn do_throw(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_try(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_catch(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_finally(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_endtry(_a0:usize) { }

#[no_mangle]
pub extern "C" fn enter_cleanup(_a0:usize) { }

#[no_mangle]
pub extern "C" fn leave_cleanup(_a0:usize) { }

#[no_mangle]
pub extern "C" fn cleanup_conditionals(_a0:usize, _a1:usize, _a2:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn rewind_conditionals(_a0:usize, _a1:usize, _a2:usize, _a3:usize) { }

#[no_mangle]
pub extern "C" fn ex_endfunction(_a0:usize) { }

#[no_mangle]
pub extern "C" fn has_loop_cmd(_a0:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn parse_pattern_and_range(_a0:usize, _a1:usize, _a2:usize, _a3:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn cmdline_init() { }

#[no_mangle]
pub extern "C" fn getcmdline(_a0:usize, _a1:usize, _a2:usize, _a3:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn getcmdline_prompt(_a0:usize, _a1:usize, _a2:usize, _a3:usize, _a4:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn check_opt_wim() -> c_int { 0 }

#[no_mangle]
pub extern "C" fn text_locked() -> c_int { 0 }

#[no_mangle]
pub extern "C" fn text_locked_msg() { }

#[no_mangle]
pub extern "C" fn get_text_locked_msg() -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn text_or_buf_locked() -> c_int { 0 }

#[no_mangle]
pub extern "C" fn curbuf_locked() -> c_int { 0 }

#[no_mangle]
pub extern "C" fn allbuf_locked() -> c_int { 0 }

#[no_mangle]
pub extern "C" fn getexline(_a0:usize, _a1:usize, _a2:usize, _a3:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn getexmodeline(_a0:usize, _a1:usize, _a2:usize, _a3:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn cmdline_overstrike() -> c_int { 0 }

#[no_mangle]
pub extern "C" fn cmdline_at_end() -> c_int { 0 }

#[no_mangle]
pub extern "C" fn cmdline_getvcol_cursor() -> c_int { 0 }

#[no_mangle]
pub extern "C" fn realloc_cmdbuff(_a0:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn free_arshape_buf() { }

#[no_mangle]
pub extern "C" fn putcmdline(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn unputcmdline() { }

#[no_mangle]
pub extern "C" fn put_on_cmdline(_a0:usize, _a1:usize, _a2:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn cmdline_paste_str(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn redrawcmdline() { }

#[no_mangle]
pub extern "C" fn redrawcmdline_ex(_a0:usize) { }

#[no_mangle]
pub extern "C" fn redrawcmd() { }

#[no_mangle]
pub extern "C" fn compute_cmdrow() { }

#[no_mangle]
pub extern "C" fn cursorcmd() { }

#[no_mangle]
pub extern "C" fn gotocmdline(_a0:usize) { }

#[no_mangle]
pub extern "C" fn vim_strsave_fnameescape(_a0:usize, _a1:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn escape_fname(_a0:usize) { }

#[no_mangle]
pub extern "C" fn tilde_replace(_a0:usize, _a1:usize, _a2:usize) { }

#[no_mangle]
pub extern "C" fn get_cmdline_info() -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn f_getcmdcomplpat(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn f_getcmdcompltype(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn f_getcmdline(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn f_getcmdpos(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn f_getcmdprompt(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn f_getcmdscreenpos(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn f_getcmdtype(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn f_setcmdline(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn f_setcmdpos(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn get_cmdline_firstc() -> c_int { 0 }

#[no_mangle]
pub extern "C" fn get_list_range(_a0:usize, _a1:usize, _a2:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn did_set_cedit(_a0:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn is_in_cmdwin() -> c_int { 0 }

#[no_mangle]
pub extern "C" fn script_get(_a0:usize, _a1:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn get_user_input(_a0:usize, _a1:usize, _a2:usize, _a3:usize) { }

#[no_mangle]
pub extern "C" fn f_wildtrigger(_a0:usize, _a1:usize) { }

