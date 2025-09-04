use libc::c_int;

#[no_mangle]
pub extern "C" fn vim_isspace(x: c_int) -> c_int {
    ((x >= 9 && x <= 13) || x == 32) as c_int
}
