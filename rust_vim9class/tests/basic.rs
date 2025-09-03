use std::ffi::CString;

#[test]
fn ffi_evaluates_class() {
    let src = CString::new("class Bar").unwrap();
    let res = rust_vim9class::rs_vim9class_eval(src.as_ptr());
    assert_eq!(res, 3);
}
