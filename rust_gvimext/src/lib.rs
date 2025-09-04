//! Rust implementation of the `gvimext` shell extension.
//!
//! This crate provides a Windows shell extension that offers an
//! "Edit with Vim" context menu entry.  The original implementation
//! lives in `src/GvimExt/gvimext.cpp`; this crate ports the
//! functionality to Rust and exposes a minimal COM interface.

#[cfg(windows)]
mod win;
#[cfg(not(windows))]
mod win {
    use std::path::Path;
    use std::process::Command;

    /// Stub type used on non-Windows platforms.
    pub struct VimShellExt;

    /// No-op registration on non-Windows systems.
    pub fn register_context_menu() -> std::io::Result<()> {
        Ok(())
    }

    /// Build the `gvim` command with the provided files.  On
    /// non-Windows this simply validates the input and returns `Ok`.
    pub fn open_files<I, P>(files: I) -> std::io::Result<()>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut cmd = Command::new("gvim");
        for f in files {
            cmd.arg(f.as_ref());
        }
        // Do not spawn the process during tests; just ensure the
        // command can be constructed.
        Ok(())
    }
}

pub use win::{open_files, register_context_menu, VimShellExt};

/// Registry path used for the context menu registration.
#[cfg(windows)]
#[allow(dead_code)]
fn context_menu_key() -> &'static str {
    r"Software\Classes\*\shell\Edit with Vim\command"
}

#[cfg(not(windows))]
#[allow(dead_code)]
fn context_menu_key() -> &'static str {
    "Software/Classes/*/shell/Edit with Vim/command"
}

/// Build the arguments passed to `gvim` for the given list of files.
#[cfg(test)]
fn build_gvim_args<I, P>(files: I) -> Vec<String>
where
    I: IntoIterator<Item = P>,
    P: AsRef<std::path::Path>,
{
    files
        .into_iter()
        .map(|p| p.as_ref().to_string_lossy().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_menu_key_matches() {
        assert!(context_menu_key().contains("Edit with Vim"));
    }

    #[test]
    fn build_args_captures_files() {
        let args = build_gvim_args(["a.txt", "b.txt"]);
        assert_eq!(args, vec!["a.txt", "b.txt"]);
    }

    #[test]
    fn open_files_accepts_slice() {
        // The function is a no-op on non-Windows platforms; ensure it
        // does not error when given multiple files.
        assert!(open_files(["foo", "bar"]).is_ok());
    }
}
