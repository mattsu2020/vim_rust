" Tests for rust_regex_match() function provided by Rust regex integration

func Test_rust_regex_match()
  if !exists('*rust_regex_match')
    throw 'rust_regex_match not available'
  endif
  call assert_equal(1, rust_regex_match('a.c', 'abc'))
  call assert_equal(0, rust_regex_match('a.c', 'abc', 0))
  call assert_equal(0, rust_regex_match('a', 'a', 1, 0))
endfunc
