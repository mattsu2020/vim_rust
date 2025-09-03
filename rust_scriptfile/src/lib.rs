use std::os::raw::{c_char, c_int, c_long, c_uchar, c_void};

#[no_mangle]
pub extern "C" fn estack_init() {}
#[no_mangle]
pub extern "C" fn estack_push(_type: c_int, _name: *mut c_uchar, _lnum: c_long) -> *mut c_void { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn estack_push_ufunc(_ufunc: *mut c_void, _lnum: c_long) -> *mut c_void { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn estack_top_is_ufunc(_ufunc: *mut c_void, _lnum: c_long) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn estack_pop() -> *mut c_void { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn estack_sfile(_which: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn stacktrace_create() -> *mut c_void { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn f_getstacktrace(_argvars: *mut c_void, _rettv: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_runtime(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn set_context_in_runtime_cmd(_xp: *mut c_void, _arg: *mut c_uchar) {}
#[no_mangle]
pub extern "C" fn find_script_by_name(_name: *mut c_uchar) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn get_new_scriptitem_for_fname(_error: *mut c_void, _fname: *mut c_uchar) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn check_script_symlink(_sid: c_int) {}
#[no_mangle]
pub extern "C" fn do_in_path(_path: *mut c_uchar, _prefix: *mut c_char, _name: *mut c_uchar, _flags: c_int, _fname: *mut c_void, _cookie: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn do_in_runtimepath(_name: *mut c_uchar, _flags: c_int, _fname: *mut c_void, _cookie: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn source_runtime(_name: *mut c_uchar, _flags: c_int) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn source_in_path(_path: *mut c_uchar, _name: *mut c_uchar, _flags: c_int, _ret_sid: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn find_script_in_rtp(_name: *mut c_uchar) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn add_pack_start_dirs() {}
#[no_mangle]
pub extern "C" fn load_start_packages() {}
#[no_mangle]
pub extern "C" fn ex_packloadall(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_packadd(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn remove_duplicates(_gap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ExpandRTDir(_pat: *mut c_uchar, _flags: c_int, _num_file: *mut c_void, _file: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn expand_runtime_cmd(_pat: *mut c_uchar, _numMatches: *mut c_void, _matches: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn ExpandPackAddDir(_pat: *mut c_uchar, _num_file: *mut c_void, _file: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn ex_source(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_options(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn source_breakpoint(_cookie: *mut c_void) -> *mut c_void { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn source_dbg_tick(_cookie: *mut c_void) -> *mut c_void { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn source_level(_cookie: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn source_nextline(_cookie: *mut c_void) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn do_source(_fname: *mut c_uchar, _check_other: c_int, _is_vimrc: c_int, _ret_sid: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn ex_scriptnames(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn scriptnames_slash_adjust() {}
#[no_mangle]
pub extern "C" fn get_scriptname(_id: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn free_scriptnames() {}
#[no_mangle]
pub extern "C" fn free_autoload_scriptnames() {}
#[no_mangle]
pub extern "C" fn get_sourced_lnum(_cookie: *mut c_void) -> c_long { 0 }
#[no_mangle]
pub extern "C" fn f_getscriptinfo(_argvars: *mut c_void, _rettv: *mut c_void) {}
#[no_mangle]
pub extern "C" fn getsourceline(_c: c_int, _cookie: *mut c_void, _indent: c_int, _options: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn sourcing_a_script(_eap: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn ex_scriptencoding(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_scriptversion(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_finish(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn do_finish(_eap: *mut c_void, _reanimate: c_int) {}
#[no_mangle]
pub extern "C" fn source_finished(_cookie: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn get_autoload_prefix(_si: *mut c_void) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn may_prefix_autoload(_name: *mut c_uchar) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn autoload_name(_name: *mut c_uchar) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn script_autoload(_name: *mut c_uchar, _reload: c_int) -> c_int { 0 }
