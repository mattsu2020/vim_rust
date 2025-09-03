use libc::{time, time_t};

#[cfg(not(test))]
extern "C" {
    static mut time_for_testing: time_t;
}

#[cfg(test)]
#[no_mangle]
pub static mut time_for_testing: time_t = 0;

#[no_mangle]
pub unsafe extern "C" fn vim_time() -> time_t {
    if time_for_testing == 0 {
        time(std::ptr::null_mut())
    } else {
        time_for_testing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_non_zero() {
        let t = unsafe { vim_time() };
        assert!(t > 0);
    }
}
