#![allow(unexpected_cfgs)]

use std::fs::File;
use std::io::{Result, Write};

// Select the same backend types that are used by the GUI code so that
// platform specific pieces can be shared.
#[cfg(target_os = "macos")]
use rust_gui_core::backend::macos::MacBackend as Backend;
#[cfg(all(target_os = "linux", feature = "gtk"))]
use rust_gui_gtk::GtkBackend as Backend;
#[cfg(target_os = "haiku")]
use rust_gui_haiku::HaikuBackend as Backend;
#[cfg(all(target_os = "linux", feature = "motif"))]
use rust_gui_motif::MotifBackend as Backend;
#[cfg(target_os = "qnx")]
use rust_gui_photon::PhotonBackend as Backend;
#[cfg(target_os = "windows")]
use rust_gui_w32::W32Backend as Backend;
#[cfg(all(target_os = "linux", not(feature = "gtk"), not(feature = "motif")))]
use rust_gui_x11::X11Backend as Backend;

/// Trait for backend specific preparation and finalisation steps.
pub trait PrintBackend {
    fn prepare<W: Write>(&mut self, _w: &mut W) -> Result<()> {
        Ok(())
    }
    fn finish<W: Write>(&mut self, _w: &mut W) -> Result<()> {
        Ok(())
    }
}

// Provide a no-op implementation for the selected backend.  Individual
// backends can extend this in their respective crates when needed.
impl PrintBackend for Backend {}

/// Basic printer that generates a tiny PostScript file from text.
#[derive(Default)]
pub struct Hardcopy<B: PrintBackend> {
    backend: B,
}

impl<B: PrintBackend> Hardcopy<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Print `text` to `path` as a minimal PostScript document.
    pub fn print_to_file(&mut self, text: &str, path: &str) -> Result<()> {
        let mut file = File::create(path)?;
        self.backend.prepare(&mut file)?;
        writeln!(file, "%!PS-Adobe-3.0")?;
        writeln!(file, "%%Pages: 1")?;
        writeln!(file, "%%BeginProlog\n%%EndProlog")?;
        writeln!(file, "%%Page: 1 1")?;
        for line in text.lines() {
            writeln!(file, "({}) show", line)?;
        }
        writeln!(file, "showpage")?;
        writeln!(file, "%%EOF")?;
        self.backend.finish(&mut file)?;
        Ok(())
    }
}

/// C-callable entry point used by the legacy code.
#[no_mangle]
pub extern "C" fn rs_hardcopy_print(
    text_ptr: *const u8,
    text_len: usize,
    path_ptr: *const u8,
    path_len: usize,
) -> i32 {
    // Safety: the caller must pass valid UTF-8 pointers.
    let text =
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(text_ptr, text_len)) };
    let path =
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(path_ptr, path_len)) };
    let backend = Backend::new();
    let mut printer = Hardcopy::new(backend);
    match printer.print_to_file(text, path) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct DummyBackend;
    impl PrintBackend for DummyBackend {}

    #[test]
    fn creates_basic_postscript() {
        let path = std::env::temp_dir().join("test_rust_hardcopy.ps");
        let mut printer = Hardcopy::new(DummyBackend);
        printer
            .print_to_file("hello", path.to_str().unwrap())
            .unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("PS-Adobe"));
        assert!(content.contains("hello"));
        let _ = std::fs::remove_file(path);
    }
}
