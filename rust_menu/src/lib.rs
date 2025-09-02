use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

#[derive(Default)]
struct MenuItem {
    name: String,
    children: Vec<MenuItem>,
}

impl MenuItem {
    fn insert(&mut self, path: &[&str]) {
        if path.is_empty() {
            return;
        }
        let head = path[0];
        if let Some(child) = self.children.iter_mut().find(|c| c.name == head) {
            child.insert(&path[1..]);
        } else {
            let mut new_item = MenuItem { name: head.to_string(), children: Vec::new() };
            new_item.insert(&path[1..]);
            self.children.push(new_item);
        }
    }

    fn remove(&mut self, path: &[&str]) -> bool {
        if path.is_empty() {
            return false;
        }
        let head = path[0];
        if path.len() == 1 {
            if let Some(idx) = self.children.iter().position(|c| c.name == head) {
                self.children.remove(idx);
                return true;
            }
            return false;
        }
        if let Some(child) = self.children.iter_mut().find(|c| c.name == head) {
            let removed = child.remove(&path[1..]);
            if child.children.is_empty() {
                if let Some(idx) = self.children.iter().position(|c| c.name == head) {
                    self.children.remove(idx);
                }
            }
            return removed;
        }
        false
    }

    fn draw(&self, depth: usize) {
        for child in &self.children {
            for _ in 0..depth {
                print!("  ");
            }
            println!("{}", child.name);
            child.draw(depth + 1);
        }
    }
}

static ROOT: OnceLock<Mutex<MenuItem>> = OnceLock::new();

fn with_root<F, R>(f: F) -> R
where
    F: FnOnce(&mut MenuItem) -> R,
{
    let root_mutex = ROOT.get_or_init(|| Mutex::new(MenuItem::default()));
    let mut root = root_mutex.lock().unwrap();
    f(&mut root)
}

#[no_mangle]
pub extern "C" fn menu_rs_insert(path: *const c_char) -> i32 {
    if path.is_null() {
        return -1;
    }
    let c_str = unsafe { CStr::from_ptr(path) };
    if let Ok(s) = c_str.to_str() {
        let parts: Vec<&str> = s.split('.').collect();
        with_root(|root| root.insert(&parts));
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn menu_rs_remove(path: *const c_char) -> i32 {
    if path.is_null() {
        return -1;
    }
    let c_str = unsafe { CStr::from_ptr(path) };
    if let Ok(s) = c_str.to_str() {
        let parts: Vec<&str> = s.split('.').collect();
        let removed = with_root(|root| root.remove(&parts));
        if removed { 0 } else { -1 }
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn menu_rs_draw() {
    with_root(|root| root.draw(0));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn insert_and_remove() {
        let file_open = CString::new("File.Open").unwrap();
        let file_close = CString::new("File.Close").unwrap();
        menu_rs_insert(file_open.as_ptr());
        menu_rs_insert(file_close.as_ptr());
        menu_rs_remove(file_open.as_ptr());
        with_root(|root| {
            assert!(root.children.iter().any(|c| c.name == "File"));
            let file = root.children.iter().find(|c| c.name == "File").unwrap();
            assert_eq!(file.children.len(), 1);
            assert_eq!(file.children[0].name, "Close");
        });
    }
}
