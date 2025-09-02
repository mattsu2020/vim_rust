" Tests for rust spell FFI functions

func Test_rust_spell_basic()
  if !exists('*rust_spell_add') || !exists('*rust_spell_check')
    throw 'rust spell functions not available'
  endif
  call rust_spell_clear()
  call rust_spell_add('hello')
  call assert_equal(1, rust_spell_check('hello'))
  call assert_equal(0, rust_spell_check('world'))
  call rust_spell_add('world')
  call assert_equal(1, rust_spell_check('world'))
  call rust_spell_clear()
  call assert_equal(0, rust_spell_check('hello'))
endfunc
