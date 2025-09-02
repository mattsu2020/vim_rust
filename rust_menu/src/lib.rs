use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

#[repr(C)]
pub struct VimMenu {
    name: String,
    cmd: Option<String>,
    children: Vec<Box<VimMenu>>,
    enabled: bool,
}

impl VimMenu {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cmd: None,
            children: Vec::new(),
            enabled: true,
        }
    }
}

#[no_mangle]
pub extern "C" fn menu_new(name: *const c_char) -> *mut VimMenu {
    if name.is_null() {
        return ptr::null_mut();
    }
    let cname = unsafe { CStr::from_ptr(name) };
    Box::into_raw(Box::new(VimMenu::new(cname.to_str().unwrap_or_default())))
}

#[no_mangle]
pub extern "C" fn menu_add_child(parent: *mut VimMenu, child: *mut VimMenu) {
    if parent.is_null() || child.is_null() {
        return;
    }
    unsafe {
        (*parent).children.push(Box::from_raw(child));
    }
}

#[no_mangle]
pub extern "C" fn menu_set_command(menu: *mut VimMenu, cmd: *const c_char) {
    if menu.is_null() || cmd.is_null() {
        return;
    }
    let ccmd = unsafe { CStr::from_ptr(cmd) };
    unsafe {
        (*menu).cmd = Some(ccmd.to_string_lossy().into_owned());
    }
}

#[no_mangle]
pub extern "C" fn menu_execute(menu: *mut VimMenu) -> *const c_char {
    if menu.is_null() {
        return std::ptr::null();
    }
    unsafe {
        match &(*menu).cmd {
            Some(c) => CString::new(c.as_str()).unwrap().into_raw(),
            None => std::ptr::null(),
        }
    }
}

#[no_mangle]
pub extern "C" fn menu_free(menu: *mut VimMenu) {
    if menu.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(menu));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_execute() {
        let name = CString::new("File").unwrap();
        let cmd = CString::new(":wq").unwrap();
        let menu = menu_new(name.as_ptr());
        menu_set_command(menu, cmd.as_ptr());
        let ret = menu_execute(menu);
        assert_eq!(unsafe { CStr::from_ptr(ret).to_str().unwrap() }, ":wq");
        unsafe { drop(CString::from_raw(ret as *mut c_char)); }
        menu_free(menu);
    }

    #[test]
    fn add_child() {
        let parent = CString::new("File").unwrap();
        let child = CString::new("Save").unwrap();
        let p = menu_new(parent.as_ptr());
        let c = menu_new(child.as_ptr());
        menu_add_child(p, c);
        unsafe {
            assert_eq!((*p).children.len(), 1);
        }
        menu_free(p);
    }
}
