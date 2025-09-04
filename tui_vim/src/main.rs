// Minimal entry point for the TUI version of the editor.
//
// This binary just collects command-line arguments and forwards them to
// `rust_editor::tui::run`, where the real work happens.
fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    rust_editor::tui::run(&args)
}
