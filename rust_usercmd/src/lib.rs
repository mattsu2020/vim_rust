use libc::c_int;
#[no_mangle]
pub extern "C" fn find_ucmd(_a0:usize, _a1:usize, _a2:usize, _a3:usize, _a4:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn set_context_in_user_cmd(_a0:usize, _a1:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn set_context_in_user_cmdarg(_a0:usize, _a1:usize, _a2:usize, _a3:usize, _a4:usize, _a5:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn expand_user_command_name(_a0:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn get_user_commands(_a0:usize, _a1:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn get_user_command_name(_a0:usize, _a1:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn get_user_cmd_addr_type(_a0:usize, _a1:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn get_user_cmd_flags(_a0:usize, _a1:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn get_user_cmd_nargs(_a0:usize, _a1:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn get_user_cmd_complete(_a0:usize, _a1:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn cmdcomplete_type_to_str(_a0:usize, _a1:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn cmdcomplete_str_to_type(_a0:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn uc_fun_cmd() -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn parse_compl_arg(_a0:usize, _a1:usize, _a2:usize, _a3:usize, _a4:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn may_get_cmd_block(_a0:usize, _a1:usize, _a2:usize) -> *mut u8 { std::ptr::null_mut() }

#[no_mangle]
pub extern "C" fn ex_command(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_comclear(_a0:usize) { }

#[no_mangle]
pub extern "C" fn uc_clear(_a0:usize) { }

#[no_mangle]
pub extern "C" fn ex_delcommand(_a0:usize) { }

#[no_mangle]
pub extern "C" fn add_win_cmd_modifiers(_a0:usize, _a1:usize, _a2:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn produce_cmdmods(_a0:usize, _a1:usize, _a2:usize) -> c_int { 0 }

#[no_mangle]
pub extern "C" fn do_ucmd(_a0:usize) { }

