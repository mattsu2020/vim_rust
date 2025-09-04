use libc::{c_int, c_void, c_ulong, c_long};
use libc::c_uchar;
pub type char_u = c_uchar;
pub type guicolor_T = c_int;
pub type attrentry_T = c_void;
pub type expand_T = c_void;
pub type regmatch_T = c_void;
pub type typval_T = c_void;
use std::ptr;
#[no_mangle]
pub extern "C" fn highlight_num_groups() -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn highlight_group_name(_0: c_int) -> *mut char_u {
    ptr::null_mut()
}
#[no_mangle]
pub extern "C" fn highlight_link_id(_0: c_int) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn init_highlight(_0: c_int, _1: c_int) {}
#[no_mangle]
pub extern "C" fn load_colors(_0: *mut char_u) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn do_highlight(_0: *mut char_u, _1: c_int, _2: c_int) {}
#[no_mangle]
pub extern "C" fn free_highlight() {}
#[no_mangle]
pub extern "C" fn restore_cterm_colors() {}
#[no_mangle]
pub extern "C" fn set_normal_colors() {}
#[no_mangle]
pub extern "C" fn hl_get_font_name() -> *mut char_u {
    ptr::null_mut()
}
#[no_mangle]
pub extern "C" fn hl_set_font_name(_0: *mut char_u) {}
#[no_mangle]
pub extern "C" fn hl_set_bg_color_name(_0: *mut char_u) {}
#[no_mangle]
pub extern "C" fn hl_set_fg_color_name(_0: *mut char_u) {}
#[no_mangle]
pub extern "C" fn color_name2handle(_0: *mut char_u) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn gui_get_color_cmn(_0: *mut char_u) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn gui_get_rgb_color_cmn(_0: c_int, _1: c_int, _2: c_int) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn get_cterm_attr_idx(_0: c_int, _1: c_int, _2: c_int) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn get_tgc_attr_idx(_0: c_int, _1: c_int, _2: c_int) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn get_gui_attr_idx(_0: c_int, _1: c_int, _2: c_int) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn clear_hl_tables() {}
#[no_mangle]
pub extern "C" fn hl_combine_attr(_0: c_int, _1: c_int) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn syn_gui_attr2entry(_0: c_int) -> *mut c_void {
    ptr::null_mut()
}
#[no_mangle]
pub extern "C" fn syn_attr2attr(_0: c_int) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn syn_term_attr2entry(_0: c_int) -> *mut c_void {
    ptr::null_mut()
}
#[no_mangle]
pub extern "C" fn syn_cterm_attr2entry(_0: c_int) -> *mut c_void {
    ptr::null_mut()
}
#[no_mangle]
pub extern "C" fn highlight_has_attr(_0: c_int, _1: c_int, _2: c_int) -> *mut char_u {
    ptr::null_mut()
}
#[no_mangle]
pub extern "C" fn highlight_color(_0: c_int, _1: *mut char_u, _2: c_int) -> *mut char_u {
    ptr::null_mut()
}
#[no_mangle]
pub extern "C" fn highlight_gui_color_rgb(_0: c_int, _1: c_int) -> c_ulong {
    0
}
#[no_mangle]
pub extern "C" fn syn_list_header(_0: c_int, _1: c_int, _2: c_int) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn syn_name2id(_0: *mut char_u) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn syn_name2attr(_0: *mut char_u) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn highlight_exists(_0: *mut char_u) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn syn_id2name(_0: c_int) -> *mut char_u {
    ptr::null_mut()
}
#[no_mangle]
pub extern "C" fn syn_namen2id(_0: *mut char_u, _1: c_int) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn syn_check_group(_0: *mut char_u, _1: c_int) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn syn_id2attr(_0: c_int) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn syn_id2colors(_0: c_int, _1: *mut c_void, _2: *mut c_void) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn syn_id2cterm_bg(_0: c_int, _1: *mut c_void, _2: *mut c_void) {}
#[no_mangle]
pub extern "C" fn syn_get_final_id(_0: c_int) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn highlight_gui_started() {}
#[no_mangle]
pub extern "C" fn highlight_changed() -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn set_context_in_highlight_cmd(_0: *mut c_void, _1: *mut char_u) {}
#[no_mangle]
pub extern "C" fn get_highlight_name(_0: *mut c_void, _1: c_int) -> *mut char_u {
    ptr::null_mut()
}
#[no_mangle]
pub extern "C" fn get_highlight_name_ext(_0: *mut c_void, _1: c_int, _2: c_int) -> *mut char_u {
    ptr::null_mut()
}
#[no_mangle]
pub extern "C" fn expand_highlight_group(_0: *mut char_u, _1: *mut c_void, _2: *mut c_void, _3: *mut char_u, _4: *mut c_void) -> c_int {
    0
}
#[no_mangle]
pub extern "C" fn free_highlight_fonts() {}
#[no_mangle]
pub extern "C" fn f_hlget(_0: *mut c_void, _1: *mut c_void) {}
#[no_mangle]
pub extern "C" fn f_hlset(_0: *mut c_void, _1: *mut c_void) {}
