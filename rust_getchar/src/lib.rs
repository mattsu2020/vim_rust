use rust_input::{rs_input_avail, rs_input_get, rs_input_unget, InputContext};
use std::os::raw::{c_int, c_uint};

#[no_mangle]
pub extern "C" fn rs_getchar(ctx: *mut InputContext) -> c_int {
    rs_input_get(ctx)
}

#[no_mangle]
pub extern "C" fn rs_getchar_avail(ctx: *mut InputContext) -> c_int {
    rs_input_avail(ctx)
}

#[no_mangle]
pub extern "C" fn rs_ungetchar(ctx: *mut InputContext, key: c_uint) {
    rs_input_unget(ctx, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_input::{rs_input_context_free, rs_input_context_new, rs_input_feed};

    #[test]
    fn feed_unget_and_avail() {
        let ctx = rs_input_context_new();
        assert_eq!(rs_getchar_avail(ctx), 0);
        rs_input_feed(ctx, 'x' as u32);
        assert_eq!(rs_getchar_avail(ctx), 1);
        assert_eq!(rs_getchar(ctx), 'x' as i32);
        rs_ungetchar(ctx, 'y' as u32);
        assert_eq!(rs_getchar_avail(ctx), 1);
        assert_eq!(rs_getchar(ctx), 'y' as i32);
        rs_input_context_free(ctx);
    }
}
