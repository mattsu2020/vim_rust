use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Clone, Debug)]
struct MenuItem {
    enabled: bool,
    /// Optional key binding used to activate the menu entry.
    key: Option<String>,
    children: HashMap<String, MenuItem>,
}

impl MenuItem {
    fn new() -> Self {
        MenuItem { enabled: true, key: None, children: HashMap::new() }
    }

    fn remove_path(&mut self, parts: &[&str]) -> bool {
        if parts.len() == 1 {
            self.children.remove(parts[0]).is_some()
        } else if let Some(child) = self.children.get_mut(parts[0]) {
            let removed = child.remove_path(&parts[1..]);
            if removed && child.children.is_empty() {
                self.children.remove(parts[0]);
            }
            removed
        } else {
            false
        }
    }

    fn find_mut(&mut self, parts: &[&str]) -> Option<&mut MenuItem> {
        let mut cur = self;
        for p in parts {
            cur = cur.children.get_mut(*p)?;
        }
        Some(cur)
    }
}

static MENUS: Lazy<Mutex<MenuItem>> = Lazy::new(|| Mutex::new(MenuItem::new()));

fn cstr_to_str<'a>(s: *const c_char) -> Option<&'a str> {
    if s.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(s) }.to_str().ok()
    }
}

#[no_mangle]
pub extern "C" fn rs_menu_add(path: *const c_char) -> c_int {
    if let Some(p) = cstr_to_str(path) {
        let mut node = MENUS.lock().unwrap();
        let mut cur = &mut *node;
        for part in p.split('.') {
            cur = cur.children.entry(part.to_string()).or_insert_with(MenuItem::new);
        }
        return 1;
    }
    0
}

#[no_mangle]
pub extern "C" fn rs_menu_remove(path: *const c_char) -> c_int {
    if let Some(p) = cstr_to_str(path) {
        let parts: Vec<&str> = p.split('.').collect();
        if parts.is_empty() {
            return 0;
        }
        let mut root = MENUS.lock().unwrap();
        return root.remove_path(&parts) as c_int;
    }
    0
}

#[no_mangle]
pub extern "C" fn rs_menu_show() -> c_int {
    fn count(node: &MenuItem) -> i32 {
        node.children.values().map(|c| 1 + count(c)).sum()
    }
    let root = MENUS.lock().unwrap();
    count(&root) as c_int
}

#[no_mangle]
pub extern "C" fn rs_menu_bind_key(path: *const c_char, key: *const c_char) -> c_int {
    let (p, k) = (cstr_to_str(path), cstr_to_str(key));
    if let (Some(p), Some(k)) = (p, k) {
        let parts: Vec<&str> = p.split('.').collect();
        let mut root = MENUS.lock().unwrap();
        if let Some(item) = root.find_mut(&parts) {
            item.key = Some(k.to_string());
            return 1;
        }
    }
    0
}

fn draw_recursive(node: &MenuItem, indent: usize, out: &mut String) {
    for (name, child) in &node.children {
        for _ in 0..indent {
            out.push(' ');
        }
        out.push_str(name);
        if let Some(k) = &child.key {
            out.push_str(" [");
            out.push_str(k);
            out.push(']');
        }
        out.push('\n');
        draw_recursive(child, indent + 2, out);
    }
}

#[no_mangle]
pub extern "C" fn rs_menu_draw() -> *mut c_char {
    let root = MENUS.lock().unwrap();
    let mut out = String::new();
    draw_recursive(&root, 0, &mut out);
    CString::new(out).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn rs_menu_free_str(s: *mut c_char) {
    if !s.is_null() {
        unsafe { CString::from_raw(s); }
    }
}

// Stubs replacing former C functions. These allow linking while the menu system
// is gradually ported to Rust.

#[no_mangle]
pub extern "C" fn winbar_height(_wp: *mut c_void) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn ex_menu(_eap: *mut c_void) {}

#[no_mangle]
pub extern "C" fn remove_winbar(_wp: *mut c_void) {}

#[no_mangle]
pub extern "C" fn set_context_in_menu_cmd(_xp: *mut c_void, _cmd: *mut c_char, _arg: *mut c_char, _forceit: c_int) -> *mut c_char {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn get_menu_name(_xp: *mut c_void, _idx: c_int) -> *mut c_char {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn get_menu_names(_xp: *mut c_void, _idx: c_int) -> *mut c_char {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn get_menu_index(_menu: *mut c_void, _state: c_int) -> c_int {
    -1
}

#[no_mangle]
pub extern "C" fn menu_is_menubar(_name: *mut c_char) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn menu_is_popup(_name: *mut c_char) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn menu_is_child_of_popup(_menu: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn menu_is_toolbar(_name: *mut c_char) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn menu_is_separator(_name: *mut c_char) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn get_menu_mode_flag() -> c_int { 0 }
#[no_mangle]
pub extern "C" fn show_popupmenu() {}
#[no_mangle]
pub extern "C" fn check_menu_pointer(_root: *mut c_void, _check: *mut c_void) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn gui_create_initial_menus(_menu: *mut c_void) {}
#[no_mangle]
pub extern "C" fn gui_update_menus(_modes: c_int) {}
#[no_mangle]
pub extern "C" fn gui_is_menu_shortcut(_key: c_int) -> c_int { 0 }
#[no_mangle]
pub extern "C" fn gui_mch_toggle_tearoffs(_enable: c_int) {}
#[no_mangle]
pub extern "C" fn execute_menu(_eap: *mut c_void, _menu: *mut c_void, _mode_idx: c_int) {}
#[no_mangle]
pub extern "C" fn ex_emenu(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn winbar_click(_wp: *mut c_void, _col: c_int) {}
#[no_mangle]
pub extern "C" fn gui_find_menu(_path_name: *mut c_char) -> *mut c_void { std::ptr::null_mut() }
#[no_mangle]
pub extern "C" fn ex_menutranslate(_eap: *mut c_void) {}
#[no_mangle]
pub extern "C" fn f_menu_info(_argvars: *mut c_void, _rettv: *mut c_void) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    fn reset() {
        MENUS.lock().unwrap().children.clear();
    }

    #[test]
    fn add_and_remove() {
        reset();
        let path = std::ffi::CString::new("File.Open").unwrap();
        assert_eq!(rs_menu_add(path.as_ptr()), 1);
        assert_eq!(rs_menu_show(), 2);
        assert_eq!(rs_menu_remove(path.as_ptr()), 1);
        assert_eq!(rs_menu_show(), 0);
    }

    #[test]
    fn bind_and_draw() {
        reset();
        let path = std::ffi::CString::new("File.Save").unwrap();
        assert_eq!(rs_menu_add(path.as_ptr()), 1);
        let key = std::ffi::CString::new("S").unwrap();
        assert_eq!(rs_menu_bind_key(path.as_ptr(), key.as_ptr()), 1);
        let drawn_ptr = rs_menu_draw();
        let drawn = unsafe { CStr::from_ptr(drawn_ptr) }.to_str().unwrap().to_string();
        rs_menu_free_str(drawn_ptr);
        assert!(drawn.contains("File"));
        assert!(drawn.contains("[S]"));
    }
}
