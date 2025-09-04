use std::ffi::{CStr};
use std::os::raw::{c_char, c_int, c_long, c_ulong};
use std::sync::Mutex;

use once_cell::sync::Lazy;

#[derive(Clone)]
#[allow(dead_code)]
struct UserCommand {
    name: String,
    rep: String,
    argt: c_ulong,
    def: c_long,
    flags: c_int,
    compl: c_int,
    addr_type: c_int,
    compl_arg: Option<String>,
}

static COMMANDS: Lazy<Mutex<Vec<UserCommand>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[no_mangle]
pub extern "C" fn rs_user_command_register(
    name: *const c_char,
    rep: *const c_char,
    argt: c_ulong,
    def: c_long,
    flags: c_int,
    compl: c_int,
    compl_arg: *const c_char,
    addr_type: c_int,
    force: c_int,
) -> c_int {
    if name.is_null() || rep.is_null() {
        return -1;
    }
    let name = unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned();
    let rep = unsafe { CStr::from_ptr(rep) }.to_string_lossy().into_owned();
    let compl_arg = if compl_arg.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(compl_arg) }.to_string_lossy().into_owned())
    };

    let mut cmds = COMMANDS.lock().unwrap();
    if let Some(pos) = cmds.iter().position(|c| c.name == name) {
        if force == 0 {
            return -1;
        }
        cmds[pos] = UserCommand { name, rep, argt, def, flags, compl, addr_type, compl_arg };
    } else {
        cmds.push(UserCommand { name, rep, argt, def, flags, compl, addr_type, compl_arg });
    }
    0
}

#[no_mangle]
pub extern "C" fn rs_user_command_delete(name: *const c_char) -> c_int {
    if name.is_null() {
        return -1;
    }
    let name = unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned();
    let mut cmds = COMMANDS.lock().unwrap();
    if let Some(pos) = cmds.iter().position(|c| c.name == name) {
        cmds.remove(pos);
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn rs_user_command_clear() {
    COMMANDS.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn register_and_delete() {
        let name = CString::new("Cmd").unwrap();
        let rep = CString::new("echo").unwrap();
        let res = rs_user_command_register(
            name.as_ptr(),
            rep.as_ptr(),
            0,
            0,
            0,
            0,
            std::ptr::null(),
            0,
            0,
        );
        assert_eq!(res, 0);
        assert_eq!(rs_user_command_delete(name.as_ptr()), 0);
    }
}
