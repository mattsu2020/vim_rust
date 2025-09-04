use rust_screen::{
    rs_screen_draw_text, rs_screen_flush, rs_screen_free, rs_screen_highlight_info, rs_screen_new,
    HighlightInfo,
};
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::sync::Mutex;

static DATA: Mutex<Vec<(c_int, Vec<u8>)>> = Mutex::new(Vec::new());

extern "C" fn collect(row: c_int, _text: *const c_char, attr: *const u8, len: c_int) {
    let slice = unsafe { std::slice::from_raw_parts(attr, len as usize) };
    DATA.lock().unwrap().push((row, slice.to_vec()));
}

#[test]
fn highlight_struct_applies_theme() {
    DATA.lock().unwrap().clear();
    let sb = rs_screen_new(5, 1);
    let txt = CString::new("hello").unwrap();
    rs_screen_draw_text(sb, 0, 0, txt.as_ptr(), 1);
    let info = HighlightInfo {
        row: 0,
        col: 1,
        len: 3,
        attr: 9,
    };
    rs_screen_highlight_info(sb, &info);
    rs_screen_flush(sb, Some(collect));
    rs_screen_free(sb);
    let data = DATA.lock().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].1[1..4], vec![9, 9, 9]);
}
