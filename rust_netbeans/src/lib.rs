use std::os::raw::{c_int, c_uchar};
use std::ptr;

#[repr(C)]
pub struct channel_T {
    _private: [u8; 0],
}

#[repr(C)]
pub struct readq_T {
    rq_buffer: *mut c_uchar,
    rq_buflen: usize,
    rq_next: *mut readq_T,
    rq_prev: *mut readq_T,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum ch_part_T {
    PART_SOCK = 0,
}

extern "C" {
    fn netbeans_rs_get_channel() -> *mut channel_T;
    fn channel_peek(channel: *mut channel_T, part: ch_part_T) -> *mut readq_T;
    fn channel_first_nl(node: *mut readq_T) -> *mut c_uchar;
    fn channel_collapse(channel: *mut channel_T, part: ch_part_T, want_nl: c_int) -> c_int;
    fn channel_get(channel: *mut channel_T, part: ch_part_T, outlen: *mut c_int) -> *mut c_uchar;
    fn nb_parse_cmd_rs(cmd: *mut c_uchar);
    fn vim_free(ptr: *mut c_uchar);
    fn channel_consume(channel: *mut channel_T, part: ch_part_T, len: c_int);
}

const OK: c_int = 0;

#[no_mangle]
pub extern "C" fn rs_netbeans_parse_messages() {
    unsafe {
        let ch = netbeans_rs_get_channel();
        if ch.is_null() {
            return;
        }
        loop {
            let node = channel_peek(ch, ch_part_T::PART_SOCK);
            if node.is_null() {
                break;
            }
            let mut p = channel_first_nl(node);
            if p.is_null() {
                if channel_collapse(ch, ch_part_T::PART_SOCK, 1) != OK {
                    return;
                }
                continue;
            }
            // Terminate command at end of line
            *p = 0;
            p = p.add(1);
            let (buffer, own_node);
            if *p == 0 {
                own_node = true;
                buffer = channel_get(ch, ch_part_T::PART_SOCK, ptr::null_mut());
                if buffer.is_null() {
                    return;
                }
            } else {
                own_node = false;
                buffer = (*node).rq_buffer;
            }
            nb_parse_cmd_rs(buffer);
            if own_node {
                vim_free(buffer);
            } else if !netbeans_rs_get_channel().is_null() {
                let len = p.offset_from(buffer) as c_int;
                channel_consume(ch, ch_part_T::PART_SOCK, len);
            }
        }
    }
}
