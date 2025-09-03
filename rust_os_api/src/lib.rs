use crossterm::{cursor, terminal, ExecutableCommand};

use std::io::{stdout, Write, Result};

/// Move the cursor to the given position.
pub fn move_cursor_to(x: u16, y: u16) -> Result<()> {
    stdout().execute(cursor::MoveTo(x, y))?;
    Ok(())
}

/// Clear the entire screen.
pub fn clear_screen() -> Result<()> {
    stdout().execute(terminal::Clear(terminal::ClearType::All))?;
    Ok(())
}

/// Play a simple beep sound using the BEL character.
pub fn play_beep() -> std::io::Result<()> {
    let mut out = stdout();
    out.write_all(b"\x07")?;
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beep_works() {
        play_beep().unwrap();
    }
}
