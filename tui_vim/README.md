# tui_vim

`tui_vim` provides a minimal binary wrapper around the editor's TUI.

The `main` function simply gathers command-line arguments and forwards them to
`rust_editor::tui::run`, which launches the terminal user interface.

Run it with:

```
cargo run -p vim_tui -- [args]
```
