use vim_terminal::Terminal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut term = Terminal::new(80, 24)?;
    term.render()?;
    Ok(())
}
