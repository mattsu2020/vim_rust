use std::slice;

extern "C" {
    fn rust_nv_cmds() -> *const libc::c_int;
    fn rust_nv_cmds_size() -> libc::c_int;
}

fn main() {
    unsafe {
        let size = rust_nv_cmds_size() as usize;
        let ptr = rust_nv_cmds();
        let cmds = slice::from_raw_parts(ptr, size);
        for &cmdchar in cmds {
            let mut c = cmdchar;
            if c < 0 {
                c = -c;
            }
            println!("{}", c);
        }
    }
}
