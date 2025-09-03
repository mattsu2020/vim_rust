use std::ffi::CString;

#[test]
fn root_test_cinword() {
    let line = CString::new("while (true)").unwrap();
    let words = CString::new("if,while").unwrap();
    assert!(unsafe { rust_cindent::cin_is_cinword(line.as_ptr(), words.as_ptr()) });
}

#[test]
fn root_test_not_cinword() {
    let line = CString::new("hello world").unwrap();
    let words = CString::new("if,while").unwrap();
    assert!(!unsafe { rust_cindent::cin_is_cinword(line.as_ptr(), words.as_ptr()) });
}
