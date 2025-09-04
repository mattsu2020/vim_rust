use rust_getchar::{rs_getchar, rs_getchar_avail, rs_ungetchar};
use rust_input::{rs_input_context_free, rs_input_context_new, rs_input_feed};
use std::os::raw::c_uint;

#[test]
fn roundtrip_input() {
    let ctx = rs_input_context_new();
    rs_input_feed(ctx, 'a' as c_uint);
    assert_eq!(rs_getchar_avail(ctx), 1);
    assert_eq!(rs_getchar(ctx), 'a' as i32);
    rs_ungetchar(ctx, 'b' as c_uint);
    assert_eq!(rs_getchar(ctx), 'b' as i32);
    rs_input_context_free(ctx);
}
