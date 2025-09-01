" Tests for Rust-based search implementation

func Test_rust_incremental_search()
  if !exists('*rust_regex_match')
    throw 'rust regex not available'
  endif
  new
  set incsearch
  call setline(1, ['foo', 'bar', 'foobar'])
  call feedkeys('/fo', 'nx')
  call feedkeys("\<CR>", 'nx')
  call assert_equal('foo', getline('.'))
  bwipe!
endfunc

func Test_rust_slash_command()
  if !exists('*rust_regex_match')
    throw 'rust regex not available'
  endif
  new
  call setline(1, ['alpha', 'beta', 'gamma'])
  call feedkeys('/beta\<CR>', 'nx')
  call assert_equal(2, line('.'))
  bwipe!
endfunc
