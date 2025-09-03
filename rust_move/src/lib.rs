use std::os::raw::c_int;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Position {
    pub row: c_int,
    pub col: c_int,
}

#[no_mangle]
pub extern "C" fn rs_move_cursor(mut pos: Position, drow: c_int, dcol: c_int) -> Position {
    pos.row += drow;
    pos.col += dcol;
    if pos.row < 0 { pos.row = 0; }
    if pos.col < 0 { pos.col = 0; }
    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_within_bounds() {
        let start = Position { row: 1, col: 2 };
        let end = rs_move_cursor(start, 2, -1);
        assert_eq!(end.row, 3);
        assert_eq!(end.col, 1);
    }
}
