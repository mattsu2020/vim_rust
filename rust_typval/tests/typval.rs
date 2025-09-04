use rust_typval::{alloc_tv, free_tv, typval_T, vartype_T, TypVal};
use std::ffi::CString;

#[test]
fn alloc_and_free_string_tv() {
    unsafe {
        let tv = alloc_tv();
        (*tv).v_type = vartype_T::VAR_STRING;
        let s = CString::new("hello").unwrap();
        (*tv).vval.v_string = s.into_raw() as *mut u8;
        free_tv(tv);
    }
}

#[test]
fn typval_enum_roundtrip() {
    let val = TypVal::String("hello".to_string());
    let raw = Box::into_raw(Box::new(typval_T::from(val.clone())));
    unsafe {
        let back = TypVal::from(&*raw);
        assert_eq!(back, val);
        free_tv(raw);
    }
}
