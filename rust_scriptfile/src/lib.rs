use libc::c_int;
#[no_mangle]
pub extern "C" fn estack_init() { }

#[no_mangle]
pub extern "C" fn estack_push(_a0:usize, _a1:usize, _a2:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn estack_push_ufunc(_a0:usize, _a1:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn estack_top_is_ufunc(_a0:usize, _a1:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn estack_pop() -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn estack_sfile(_a0:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn stacktrace_create() -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn f_getstacktrace(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn ex_runtime(_a0:usize) { }

#[no_mangle]
pub extern "C" fn set_context_in_runtime_cmd(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn find_script_by_name(_a0:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn get_new_scriptitem_for_fname(_a0:usize, _a1:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn check_script_symlink(_a0:usize) { }

#[no_mangle]
pub extern "C" fn do_in_path(_a0:usize, _a1:usize, _a2:usize, _a3:usize, _a4:usize, _a5:usize, _a6:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn do_in_runtimepath(_a0:usize, _a1:usize, _a2:usize, _a3:usize, _a4:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn source_runtime(_a0:usize, _a1:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn source_in_path(_a0:usize, _a1:usize, _a2:usize, _a3:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn find_script_in_rtp(_a0:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn add_pack_start_dirs() { }

#[no_mangle]
pub extern "C" fn load_start_packages() { }

#[no_mangle]
pub extern "C" fn ex_packloadall(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_packadd(_a0:usize) { }

#[no_mangle]
pub extern "C" fn remove_duplicates(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ExpandRTDir(_a0:usize, _a1:usize, _a2:usize, _a3:usize, _a4:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn expand_runtime_cmd(_a0:usize, _a1:usize, _a2:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn ExpandPackAddDir(_a0:usize, _a1:usize, _a2:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn ex_source(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_options(_a0:usize) { }

#[no_mangle]
pub extern "C" fn source_breakpoint(_a0:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn source_dbg_tick(_a0:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn source_level(_a0:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn source_nextline(_a0:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn do_source(_a0:usize, _a1:usize, _a2:usize, _a3:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn ex_scriptnames(_a0:usize) { }

#[no_mangle]
pub extern "C" fn scriptnames_slash_adjust() { }

#[no_mangle]
pub extern "C" fn get_scriptname(_a0:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn free_scriptnames() { }

#[no_mangle]
pub extern "C" fn free_autoload_scriptnames() { }

#[no_mangle]
pub extern "C" fn get_sourced_lnum(_a0:usize, _a1:usize, _a2:usize, _a3:usize, _a4:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn f_getscriptinfo(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn getsourceline(_a0:usize, _a1:usize, _a2:usize, _a3:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn sourcing_a_script(_a0:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn ex_scriptencoding(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_scriptversion(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_finish(_a0:usize) { }

#[no_mangle]
pub extern "C" fn do_finish(_a0:usize, _a1:usize) { }

#[no_mangle]
pub extern "C" fn source_finished(_a0:usize, _a1:usize, _a2:usize, _a3:usize, _a4:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn get_autoload_prefix(_a0:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn may_prefix_autoload(_a0:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn autoload_name(_a0:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn script_autoload(_a0:usize, _a1:usize) -> c_int { 0 }

