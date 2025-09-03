use std::os::raw::c_int;

#[no_mangle]
pub extern "C" fn rs_compute_indent(level: c_int) -> c_int {
    if level < 0 {
        0
    } else {
        level * 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_indent_basic() {
        assert_eq!(rs_compute_indent(0), 0);
        assert_eq!(rs_compute_indent(1), 2);
        assert_eq!(rs_compute_indent(4), 8);
    }

    #[test]
    fn compute_indent_negative() {
        assert_eq!(rs_compute_indent(-3), 0);
    }
}
