use rust_screen::ScreenBuffer;

/// Trait implemented by rendering backends.
pub trait Renderer {
    fn draw_line(&mut self, row: usize, text: &str, attrs: &[u8]);
}

/// Renderer that prints to stdout, useful for CLI mode.
pub struct CliRenderer;

impl Renderer for CliRenderer {
    fn draw_line(&mut self, row: usize, text: &str, _attrs: &[u8]) {
        println!("{row}: {text}");
    }
}

/// High level UI abstraction over [`ScreenBuffer`].
pub struct Ui<R: Renderer> {
    screen: ScreenBuffer,
    pub renderer: R,
}

impl<R: Renderer> Ui<R> {
    pub fn new(width: usize, height: usize, renderer: R) -> Self {
        Self {
            screen: ScreenBuffer::new(width, height),
            renderer,
        }
    }

    pub fn draw_text(&mut self, row: usize, col: usize, text: &str, attr: u8) {
        self.screen.draw_text(row, col, text, attr);
    }

    pub fn highlight(&mut self, row: usize, col: usize, len: usize, attr: u8) {
        self.screen.highlight_range(row, col, len, attr);
    }

    pub fn format_text(&self, text: &str, width: usize) -> String {
        ScreenBuffer::format_text(text, width)
    }

    pub fn flush(&mut self) {
        for diff in self.screen.flush_diff() {
            self.renderer.draw_line(diff.row, &diff.text, &diff.attrs);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CollectRenderer(pub Vec<String>);

    impl Renderer for CollectRenderer {
        fn draw_line(&mut self, _row: usize, text: &str, _attrs: &[u8]) {
            self.0.push(text.to_string());
        }
    }

    #[test]
    fn cli_and_gui_render() {
        let mut ui_cli = Ui::new(10, 2, CollectRenderer(Vec::new()));
        ui_cli.draw_text(0, 0, "hi", 1);
        ui_cli.flush();
        assert_eq!(ui_cli.renderer.0[0].trim_end(), "hi");

        let mut ui_gui = Ui::new(5, 1, CollectRenderer(Vec::new()));
        ui_gui.draw_text(0, 0, "ab", 1);
        ui_gui.highlight(0, 0, 2, 2);
        ui_gui.flush();
        assert_eq!(ui_gui.renderer.0[0], "ab   ");
    }

    #[test]
    fn formatting() {
        let ui = Ui::new(5, 1, CollectRenderer(Vec::new()));
        assert_eq!(ui.format_text("abc", 5), "abc  ");
        assert_eq!(ui.format_text("abcdef", 3), "abc");
    }
}
