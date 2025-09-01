use libc::{c_char, c_int};
use std::collections::HashMap;


use std::ptr;

const MEMFILE_PAGE_SIZE: usize = 4096;

#[repr(C)]
pub struct BlockHdr {
    pub bh_data: *mut u8,
    pub bh_bnum: i64,
    pub bh_page_count: c_int,
    pub bh_flags: u8,
}

struct Block {
    hdr: BlockHdr,
    data: Vec<u8>,
}

#[repr(C)]
pub struct MemFile {
    blocks: HashMap<i64, Box<Block>>,
    next_blocknr: i64,
}

impl MemFile {
    fn new() -> Self {
        Self { blocks: HashMap::new(), next_blocknr: 0 }
    }
}

#[no_mangle]
pub extern "C" fn rs_mf_open(_fname: *const c_char, _flags: c_int) -> *mut MemFile {
    Box::into_raw(Box::new(MemFile::new()))
}

#[no_mangle]
pub extern "C" fn rs_mf_new(mfp: *mut MemFile, negative: c_int, page_count: c_int) -> *mut BlockHdr {
    if mfp.is_null() { return ptr::null_mut(); }
    let mf = unsafe { &mut *mfp };
    let size = MEMFILE_PAGE_SIZE * page_count.max(1) as usize;
    let mut data = vec![0u8; size];
    let ptr_data = data.as_mut_ptr();
    let bnum = if negative != 0 { -(mf.next_blocknr + 1) } else { mf.next_blocknr };
    mf.next_blocknr += 1;
    let block = Box::new(Block {
        hdr: BlockHdr { bh_data: ptr_data, bh_bnum: bnum, bh_page_count: page_count, bh_flags: 0 },
        data,
    });
    let hdr_ptr = &block.hdr as *const BlockHdr as *mut BlockHdr;
    mf.blocks.insert(bnum, block);
    hdr_ptr
}

#[no_mangle]
pub extern "C" fn rs_mf_get(mfp: *mut MemFile, nr: i64, _page_count: c_int) -> *mut BlockHdr {
    if mfp.is_null() { return ptr::null_mut(); }
    let mf = unsafe { &mut *mfp };
    if let Some(block) = mf.blocks.get_mut(&nr) {
        block.hdr.bh_data = block.data.as_mut_ptr();
        return &mut block.hdr as *mut BlockHdr;
    }
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn rs_mf_put(_mfp: *mut MemFile, hp: *mut BlockHdr, dirty: c_int, _infile: c_int) {
    if hp.is_null() { return; }
    let hdr = unsafe { &mut *hp };
    if dirty != 0 { hdr.bh_flags |= 1; }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn create_and_use_blocks() {
        let mf = rs_mf_open(ptr::null(), 0);
        assert!(!mf.is_null());
        let bh1 = rs_mf_new(mf, 0, 1);
        assert!(!bh1.is_null());
        let hdr1 = unsafe { &mut *bh1 };
        unsafe { std::ptr::write_bytes(hdr1.bh_data, b'a', MEMFILE_PAGE_SIZE); }
        rs_mf_put(mf, bh1, 1, 0);
        let got = rs_mf_get(mf, hdr1.bh_bnum, 1);
        assert_eq!(got, bh1);
        unsafe { drop(Box::from_raw(mf)); } // clean up
    }
}
