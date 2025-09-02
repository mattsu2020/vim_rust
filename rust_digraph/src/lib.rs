mod digraph_table;
use digraph_table::{DIGRAPHS, Digr, DIGRAPH_TABLE_LEN};
use libc::c_int;

#[no_mangle]
pub extern "C" fn rs_digraph_lookup(char1: c_int, char2: c_int) -> c_int {
    for d in DIGRAPHS.iter() {
        if d.char1 as c_int == char1 && d.char2 as c_int == char2 {
            return d.result as c_int;
        }
        if d.char1 == 0 && d.char2 == 0 {
            break;
        }
    }
    0
}

#[no_mangle]
pub static digraphdefault: [Digr; DIGRAPH_TABLE_LEN] = DIGRAPHS;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lookup_known() {
        assert_eq!(rs_digraph_lookup('0' as c_int, '0' as c_int), 0x221E);
        assert_eq!(rs_digraph_lookup('A' as c_int, '!' as c_int), 0xC0);
    }
}
