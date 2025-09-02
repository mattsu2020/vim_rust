use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::rc::{Rc, Weak};

#[derive(Default)]
struct Value {
    data: i64,
}

#[derive(Default)]
struct Dict {
    items: HashMap<String, Rc<RefCell<Value>>>,
}

thread_local! {
    static GC_LIST: RefCell<Vec<Weak<RefCell<Dict>>>> = RefCell::new(Vec::new());
}

#[no_mangle]
pub extern "C" fn rust_dict_new() -> *mut c_void {
    let dict = Rc::new(RefCell::new(Dict::default()));
    GC_LIST.with(|list| list.borrow_mut().push(Rc::downgrade(&dict)));
    Box::into_raw(Box::new(dict)) as *mut c_void
}

#[no_mangle]
pub extern "C" fn rust_dict_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let boxed: Box<Rc<RefCell<Dict>>> = Box::from_raw(ptr as *mut Rc<RefCell<Dict>>);
        GC_LIST.with(|list| {
            list.borrow_mut()
                .retain(|w| w.upgrade().map_or(false, |rc| !Rc::ptr_eq(&rc, &boxed)));
        });
        drop(boxed);
    }
}

unsafe fn to_key(key: *const c_char) -> Option<String> {
    if key.is_null() {
        return None;
    }
    CStr::from_ptr(key).to_str().ok().map(|s| s.to_owned())
}

#[no_mangle]
pub extern "C" fn rust_dict_set(
    ptr: *mut c_void,
    key: *const c_char,
    value: i64,
) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    let key = match unsafe { to_key(key) } {
        Some(k) => k,
        None => return 0,
    };
    let rc = unsafe { &*(ptr as *const Rc<RefCell<Dict>>) };
    let mut dict = rc.borrow_mut();
    let val = Rc::new(RefCell::new(Value { data: value }));
    dict.items.insert(key, val);
    1
}

#[no_mangle]
pub extern "C" fn rust_dict_get(
    ptr: *mut c_void,
    key: *const c_char,
    out_value: *mut i64,
) -> c_int {
    if ptr.is_null() || out_value.is_null() {
        return 0;
    }
    let key = match unsafe { to_key(key) } {
        Some(k) => k,
        None => return 0,
    };
    let rc = unsafe { &*(ptr as *const Rc<RefCell<Dict>>) };
    let dict = rc.borrow();
    if let Some(v) = dict.items.get(&key) {
        let val = v.borrow().data;
        unsafe {
            *out_value = val;
        }
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn rust_dict_gc_collect() {
    GC_LIST.with(|list| {
        list.borrow_mut().retain(|w| w.upgrade().is_some());
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn basic_operations() {
        let dict = rust_dict_new();
        let key = CString::new("alpha").unwrap();
        assert_eq!(rust_dict_set(dict, key.as_ptr(), 42), 1);
        let mut out = 0;
        assert_eq!(rust_dict_get(dict, key.as_ptr(), &mut out), 1);
        assert_eq!(out, 42);
        rust_dict_free(dict);
        rust_dict_gc_collect();
    }
}
