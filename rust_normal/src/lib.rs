use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr;

#[repr(C)]
pub struct oparg_T {
    _private: [u8; 0],
}

#[cfg(not(test))]
extern "C" {
    fn normal_cmd_c(oap: *mut oparg_T, toplevel: c_int);
    static mut p_sc: c_int;
    static mut showcmd_buf: [c_char; SHOWCMD_BUFLEN];
    static mut msg_silent: c_int;
    static mut showcmd_visual: c_int;
    fn char_avail() -> c_int;
    fn display_showcmd();
    fn transchar(c: c_int) -> *mut c_char;
    fn mb_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn vim_isprintc(c: c_int) -> c_int;
    fn setcursor();
}

#[cfg(test)]
#[no_mangle]
extern "C" fn normal_cmd_c(_oap: *mut oparg_T, _toplevel: c_int) {}

const SHOWCMD_BUFLEN: usize = 41;
const SHOWCMD_COLS: c_int = 10;
const MB_MAXBYTES: usize = 21;

#[cfg(test)]
#[no_mangle]
static mut p_sc: c_int = 0;
#[cfg(test)]
#[no_mangle]
static mut showcmd_buf: [c_char; SHOWCMD_BUFLEN] = [0; SHOWCMD_BUFLEN];
#[cfg(test)]
#[no_mangle]
static mut msg_silent: c_int = 0;
#[cfg(test)]
#[no_mangle]
static mut showcmd_visual: c_int = 0;
#[cfg(test)]
#[no_mangle]
extern "C" fn char_avail() -> c_int {
    0
}
#[cfg(test)]
#[no_mangle]
extern "C" fn display_showcmd() {}
#[cfg(test)]
static mut TRANSCHAR_BUF: [c_char; MB_MAXBYTES + 1] = [0; MB_MAXBYTES + 1];
#[cfg(test)]
#[no_mangle]
extern "C" fn transchar(c: c_int) -> *mut c_char {
    unsafe {
        TRANSCHAR_BUF[0] = c as c_char;
        TRANSCHAR_BUF[1] = 0;
        TRANSCHAR_BUF.as_mut_ptr()
    }
}
#[cfg(test)]
#[no_mangle]
extern "C" fn mb_char2bytes(c: c_int, buf: *mut c_char) -> c_int {
    unsafe {
        *buf = c as c_char;
        1
    }
}
#[cfg(test)]
#[no_mangle]
extern "C" fn vim_isprintc(_c: c_int) -> c_int {
    1
}
#[cfg(test)]
#[no_mangle]
extern "C" fn setcursor() {}

#[no_mangle]
pub extern "C" fn normal_cmd(oap: *mut oparg_T, toplevel: c_int) {
    unsafe {
        normal_cmd_c(oap, toplevel);
    }
}

#[no_mangle]
pub extern "C" fn del_from_showcmd(len: c_int) {
    unsafe {
        if p_sc == 0 {
            return;
        }

        let mut len = len;
        let old_len = CStr::from_ptr(showcmd_buf.as_ptr()).to_bytes().len() as c_int;
        if len > old_len {
            len = old_len;
        }
        *showcmd_buf
            .as_mut_ptr()
            .add((old_len - len) as usize) = 0;

        if char_avail() == 0 {
            display_showcmd();
        }
    }
}

const KS_VER_SCROLLBAR: c_int = 249;
const KS_HOR_SCROLLBAR: c_int = 248;
const KS_EXTRA: c_int = 253;
const KE_FILLER: c_int = b'X' as c_int;
const KE_LEFTMOUSE_NM: c_int = 69;
const KE_LEFTRELEASE_NM: c_int = 70;
const KE_IGNORE: c_int = 53;
const KE_LEFTMOUSE: c_int = 44;
const KE_LEFTDRAG: c_int = 45;
const KE_LEFTRELEASE: c_int = 46;
const KE_MOUSEMOVE: c_int = 100;
const KE_MIDDLEMOUSE: c_int = 47;
const KE_MIDDLEDRAG: c_int = 48;
const KE_MIDDLERELEASE: c_int = 49;
const KE_RIGHTMOUSE: c_int = 50;
const KE_RIGHTDRAG: c_int = 51;
const KE_RIGHTRELEASE: c_int = 52;
const KE_MOUSEDOWN: c_int = 75;
const KE_MOUSEUP: c_int = 76;
const KE_MOUSELEFT: c_int = 77;
const KE_MOUSERIGHT: c_int = 78;
const KE_X1MOUSE: c_int = 89;
const KE_X1DRAG: c_int = 90;
const KE_X1RELEASE: c_int = 91;
const KE_X2MOUSE: c_int = 92;
const KE_X2DRAG: c_int = 93;
const KE_X2RELEASE: c_int = 94;
const KE_CURSORHOLD: c_int = 96;

const fn termcap2key(a: c_int, b: c_int) -> c_int {
    -(a + (b << 8))
}

const K_VER_SCROLLBAR: c_int = termcap2key(KS_VER_SCROLLBAR, KE_FILLER);
const K_HOR_SCROLLBAR: c_int = termcap2key(KS_HOR_SCROLLBAR, KE_FILLER);
const K_LEFTMOUSE_NM: c_int = termcap2key(KS_EXTRA, KE_LEFTMOUSE_NM);
const K_LEFTRELEASE_NM: c_int = termcap2key(KS_EXTRA, KE_LEFTRELEASE_NM);
const K_IGNORE: c_int = termcap2key(KS_EXTRA, KE_IGNORE);
const K_PS: c_int = termcap2key('P' as c_int, 'S' as c_int);
const K_LEFTMOUSE: c_int = termcap2key(KS_EXTRA, KE_LEFTMOUSE);
const K_LEFTDRAG: c_int = termcap2key(KS_EXTRA, KE_LEFTDRAG);
const K_LEFTRELEASE: c_int = termcap2key(KS_EXTRA, KE_LEFTRELEASE);
const K_MOUSEMOVE: c_int = termcap2key(KS_EXTRA, KE_MOUSEMOVE);
const K_MIDDLEMOUSE: c_int = termcap2key(KS_EXTRA, KE_MIDDLEMOUSE);
const K_MIDDLEDRAG: c_int = termcap2key(KS_EXTRA, KE_MIDDLEDRAG);
const K_MIDDLERELEASE: c_int = termcap2key(KS_EXTRA, KE_MIDDLERELEASE);
const K_RIGHTMOUSE: c_int = termcap2key(KS_EXTRA, KE_RIGHTMOUSE);
const K_RIGHTDRAG: c_int = termcap2key(KS_EXTRA, KE_RIGHTDRAG);
const K_RIGHTRELEASE: c_int = termcap2key(KS_EXTRA, KE_RIGHTRELEASE);
const K_MOUSEDOWN: c_int = termcap2key(KS_EXTRA, KE_MOUSEDOWN);
const K_MOUSEUP: c_int = termcap2key(KS_EXTRA, KE_MOUSEUP);
const K_MOUSELEFT: c_int = termcap2key(KS_EXTRA, KE_MOUSELEFT);
const K_MOUSERIGHT: c_int = termcap2key(KS_EXTRA, KE_MOUSERIGHT);
const K_X1MOUSE: c_int = termcap2key(KS_EXTRA, KE_X1MOUSE);
const K_X1DRAG: c_int = termcap2key(KS_EXTRA, KE_X1DRAG);
const K_X1RELEASE: c_int = termcap2key(KS_EXTRA, KE_X1RELEASE);
const K_X2MOUSE: c_int = termcap2key(KS_EXTRA, KE_X2MOUSE);
const K_X2DRAG: c_int = termcap2key(KS_EXTRA, KE_X2DRAG);
const K_X2RELEASE: c_int = termcap2key(KS_EXTRA, KE_X2RELEASE);
const K_CURSORHOLD: c_int = termcap2key(KS_EXTRA, KE_CURSORHOLD);

const IGNORE: [c_int; 28] = [
    K_VER_SCROLLBAR,
    K_HOR_SCROLLBAR,
    K_LEFTMOUSE_NM,
    K_LEFTRELEASE_NM,
    K_IGNORE,
    K_PS,
    K_LEFTMOUSE,
    K_LEFTDRAG,
    K_LEFTRELEASE,
    K_MOUSEMOVE,
    K_MIDDLEMOUSE,
    K_MIDDLEDRAG,
    K_MIDDLERELEASE,
    K_RIGHTMOUSE,
    K_RIGHTDRAG,
    K_RIGHTRELEASE,
    K_MOUSEDOWN,
    K_MOUSEUP,
    K_MOUSELEFT,
    K_MOUSERIGHT,
    K_X1MOUSE,
    K_X1DRAG,
    K_X1RELEASE,
    K_X2MOUSE,
    K_X2DRAG,
    K_X2RELEASE,
    K_CURSORHOLD,
    0,
];

#[no_mangle]
pub extern "C" fn add_to_showcmd(c: c_int) -> c_int {
    unsafe {
        if p_sc == 0 || msg_silent != 0 {
            return 0;
        }

        if showcmd_visual != 0 {
            showcmd_buf[0] = 0;
            showcmd_visual = 0;
        }

        if c < 0 {
            for &ign in IGNORE.iter() {
                if ign == 0 {
                    break;
                }
                if ign == c {
                    return 0;
                }
            }
        }

        let mut mbyte_buf = [0 as c_char; MB_MAXBYTES + 1];
        let p: *mut c_char = if c <= 0x7f || vim_isprintc(c) == 0 {
            let p = transchar(c);
            if *p == b' ' as c_char {
                ptr::copy_nonoverlapping(b"<20>\0".as_ptr() as *const c_char, p, 5);
            }
            p
        } else {
            let len = mb_char2bytes(c, mbyte_buf.as_mut_ptr());
            mbyte_buf[len as usize] = 0;
            mbyte_buf.as_mut_ptr()
        };

        let old_len = CStr::from_ptr(showcmd_buf.as_ptr()).to_bytes().len() as c_int;
        let extra_len = CStr::from_ptr(p).to_bytes().len() as c_int;
        let overflow = old_len + extra_len - SHOWCMD_COLS;
        if overflow > 0 {
            let src = showcmd_buf.as_ptr().add(overflow as usize);
            let dst = showcmd_buf.as_mut_ptr();
            let count = (old_len - overflow + 1) as usize;
            ptr::copy(src, dst, count);
        }
        let dest_len = (old_len - overflow.max(0)) as usize;
        ptr::copy_nonoverlapping(
            p,
            showcmd_buf.as_mut_ptr().add(dest_len),
            extra_len as usize + 1,
        );

        if char_avail() != 0 {
            return 0;
        }
        display_showcmd();
        1
    }
}

#[no_mangle]
pub extern "C" fn add_to_showcmd_c(c: c_int) {
    unsafe {
        if add_to_showcmd(c) == 0 {
            setcursor();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        unsafe { normal_cmd(std::ptr::null_mut(), 0) };
    }

    #[test]
    fn del_from_showcmd_basic() {
        unsafe {
            p_sc = 1;
            let initial = b"abcd\0";
            ptr::copy_nonoverlapping(
                initial.as_ptr() as *const c_char,
                showcmd_buf.as_mut_ptr(),
                initial.len(),
            );
            del_from_showcmd(2);
            let res = CStr::from_ptr(showcmd_buf.as_ptr())
                .to_str()
                .unwrap();
            assert_eq!(res, "ab");
        }
    }

    #[test]
    fn add_to_showcmd_basic() {
        unsafe {
            p_sc = 1;
            msg_silent = 0;
            showcmd_visual = 0;
            showcmd_buf[0] = 0;
            add_to_showcmd('x' as c_int);
            let res = CStr::from_ptr(showcmd_buf.as_ptr())
                .to_str()
                .unwrap();
            assert_eq!(res, "x");
        }
    }
}

