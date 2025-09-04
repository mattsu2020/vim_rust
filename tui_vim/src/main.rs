fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    rust_editor::tui::run(&args)
}

