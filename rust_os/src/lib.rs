#[cfg(unix)]
pub mod unix {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    /// Set file mode using Unix-specific API.
    pub fn set_mode(path: &Path, mode: u32) -> std::io::Result<()> {
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(mode);
        fs::set_permissions(path, perms)
    }
}

#[cfg(windows)]
pub mod win32 {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    /// Convert an `OsStr` to a null-terminated wide string.
    pub fn to_wide(s: &OsStr) -> Vec<u16> {
        s.encode_wide().chain(std::iter::once(0)).collect()
    }
}
