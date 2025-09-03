use std::os::raw::{c_char, c_int, c_long, c_uchar, c_void};

#[no_mangle]
pub extern "C" fn find_ucmd(_eap: *mut c_void, _p: *mut c_uchar, _full: *mut c_void, _xp: *mut c_void, _complp: *mut c_void) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn set_context_in_user_cmd(_xp: *mut c_void, _arg_in: *mut c_uchar) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn set_context_in_user_cmdarg(_cmd: *mut c_uchar, _arg: *mut c_uchar, _argt: c_long, _context: c_int, _xp: *mut c_void, _forceit: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn expand_user_command_name(_idx: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn get_user_commands(_xp: *mut c_void, _idx: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn get_user_command_name(_idx: c_int, _cmdidx: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn get_user_cmd_addr_type(_xp: *mut c_void, _idx: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn get_user_cmd_flags(_xp: *mut c_void, _idx: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn get_user_cmd_nargs(_xp: *mut c_void, _idx: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn get_user_cmd_complete(_xp: *mut c_void, _idx: c_int) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn cmdcomplete_type_to_str(_expand: c_int, _compl_arg: *mut c_uchar) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn cmdcomplete_str_to_type(_complete_str: *mut c_uchar) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn uc_fun_cmd() -> *mut c_char { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn parse_compl_arg(_value: *mut c_uchar, _vallen: c_int, _complp: *mut c_void, _argt: *mut c_void, _compl_arg: *mut *mut c_uchar) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn may_get_cmd_block(_eap: *mut c_void, _p: *mut c_uchar, _tofree: *mut *mut c_uchar) -> *mut c_uchar { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn ex_command(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_comclear(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn uc_clear(_gap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn ex_delcommand(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn add_win_cmd_modifiers(_buf: *mut c_uchar, _cmod: *mut c_void, _multi_mods: *mut c_void) -> usize { 0 }
#[no_mangle]
pub extern "C" fn produce_cmdmods(_buf: *mut c_uchar, _cmod: *mut c_void, _quote: c_int) -> usize { 0 }
#[no_mangle]
pub extern "C" fn do_ucmd(_eap: *mut c_void) {}
