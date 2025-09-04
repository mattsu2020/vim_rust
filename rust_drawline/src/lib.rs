/// Utilities translated from the legacy `drawline.c` file.
///
/// Currently only a small helper is implemented which advances a
/// pointer to the next color column to draw on the screen.

/// Advance the color column pointer until it is at or beyond `vcol`.
///
/// The slice pointed to by `color_cols` is expected to be terminated by
/// a negative value, matching the convention used in the original C code.
/// Returns `true` if there are further columns to handle.
pub fn advance_color_col(vcol: i32, color_cols: &mut &[i32]) -> bool {
    while !color_cols.is_empty() && color_cols[0] >= 0 && vcol > color_cols[0] {
        *color_cols = &color_cols[1..];
    }
    !color_cols.is_empty() && color_cols[0] >= 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_across_columns() {
        let cols = vec![10, 20, -1];
        let mut slice: &[i32] = &cols;
        assert!(advance_color_col(5, &mut slice));
        assert_eq!(slice, &[10, 20, -1]);
        assert!(advance_color_col(15, &mut slice));
        assert_eq!(slice, &[20, -1]);
        assert!(!advance_color_col(25, &mut slice));
    }
}
