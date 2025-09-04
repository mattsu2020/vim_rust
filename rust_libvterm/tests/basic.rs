use rust_libvterm::*;

#[test]
fn parses_text_input_moves_cursor() {
    unsafe {
        let vt = vterm_new(5, 20);
        let state = vterm_obtain_state(vt);
        vterm_state_reset(state, 1);

        let input = b"hi";
        let written = vterm_input_write(vt, input.as_ptr() as *const i8, input.len());
        assert_eq!(written, input.len());

        let mut pos = VTermPos { row: 0, col: 0 };
        vterm_state_get_cursorpos(state, &mut pos);
        assert_eq!(pos.row, 0);
        assert_eq!(pos.col, 2);

        vterm_free(vt);
    }
}
