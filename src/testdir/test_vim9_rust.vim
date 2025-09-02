" Tests for rust_vim9 integration
vim9script
# This test uses libcallnr() to invoke a Rust function.

var libext = has('mac') ? 'dylib' : has('win32') ? 'dll' : 'so'
var lib = expand('../../rust_vim9/target/debug/librust_vim9.' .. libext)

def g:Test_rust_vim9_eval()
    var res = libcallnr(lib, 'vim9_eval_int', '1 + 2 * 3')
    assert_equal(7, res)
enddef
