use std::path::Path;
#[cfg(windows)]
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(windows)]
use windows::core::{implement, HSTRING, PCIDLIST_ABSOLUTE, PWSTR};
#[cfg(windows)]
use windows::Win32::Foundation::{E_NOTIMPL, HKEY, HMENU};
#[cfg(windows)]
use windows::Win32::System::Com::IDataObject;
#[cfg(windows)]
use windows::Win32::UI::Shell::{IContextMenu, IShellExtInit, CMINVOKECOMMANDINFO};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{InsertMenuW, MF_BYPOSITION, MF_STRING};

/// Build a command to open files with gVim.
/// This mirrors a small portion of `getGvimInvocation` in the original C++.
fn build_gvim_command<I, P>(files: I) -> Command
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut cmd = Command::new("gvim");
    cmd.arg("--literal");
    for file in files {
        cmd.arg(file.as_ref());
    }
    cmd
}

/// Open the provided files with gVim.
/// On non-Windows platforms this simply spawns `gvim` if available.
pub fn open_files<I, P>(files: I) -> std::io::Result<()>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut child = build_gvim_command(files)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let _ = child.wait();
    Ok(())
}

/// Register the "Edit with Vim" context menu by writing to the registry.
#[cfg(windows)]
pub fn register_context_menu() -> windows::core::Result<()> {
    use windows::core::w;
    use windows::Win32::System::Registry::{
        RegCreateKeyExW, RegSetValueExW, HKEY_CURRENT_USER, KEY_WRITE, REG_OPTION_NON_VOLATILE,
        REG_SZ,
    }; // macro for wide strings

    unsafe {
        let mut key = HKEY::default();
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Classes\\*\\shell\\Edit with Vim\\command"),
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut key,
            None,
        )?;
        let command = HSTRING::from("gvim \"%1\"");
        RegSetValueExW(
            key,
            None,
            0,
            REG_SZ.0,
            command.as_wide().as_ptr() as *const u8,
            ((command.len() + 1) * 2) as u32,
        )?;
    }
    Ok(())
}

/// Stub on non-Windows platforms.
#[cfg(not(windows))]
pub fn register_context_menu() -> std::io::Result<()> {
    Err(std::io::Error::other(
        "context menu registration is only supported on Windows",
    ))
}

// -- COM interface implementation -----------------------------------------

#[cfg(windows)]
#[implement(IContextMenu, IShellExtInit)]
pub struct GvimExt {
    files: Vec<PathBuf>,
}

#[cfg(windows)]
impl GvimExt {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }
}

#[cfg(windows)]
impl GvimExt {
    fn menu_text() -> HSTRING {
        HSTRING::from("Edit with Vim")
    }
}

#[cfg(windows)]
impl windows::Win32::UI::Shell::IContextMenu_Impl for GvimExt {
    fn QueryContextMenu(
        &self,
        hmenu: HMENU,
        index_menu: u32,
        id_cmd_first: u32,
        _id_cmd_last: u32,
        _u_flags: u32,
    ) -> windows::core::Result<i32> {
        unsafe {
            InsertMenuW(
                hmenu,
                index_menu,
                MF_BYPOSITION | MF_STRING,
                id_cmd_first,
                windows::core::PCWSTR(Self::menu_text().as_wide().as_ptr()),
            );
        }
        Ok(1)
    }

    fn InvokeCommand(&self, _pici: *const CMINVOKECOMMANDINFO) -> windows::core::Result<()> {
        let _ = open_files(&self.files);
        Ok(())
    }

    fn GetCommandString(
        &self,
        _id_cmd: u32,
        _u_type: u32,
        _pw_reserved: *mut u32,
        _psz_name: PWSTR,
        _cch_max: u32,
    ) -> windows::core::Result<()> {
        Err(windows::core::Error::new(E_NOTIMPL, HSTRING::new()))
    }
}

#[cfg(windows)]
impl windows::Win32::UI::Shell::IShellExtInit_Impl for GvimExt {
    fn Initialize(
        &self,
        _pidl_folder: PCIDLIST_ABSOLUTE,
        _pdtobj: Option<IDataObject>,
        _hkey_prog_id: HKEY,
    ) -> windows::core::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_command_adds_literal() {
        let cmd = build_gvim_command([Path::new("file.txt")]);
        let args: Vec<_> = cmd
            .get_args()
            .map(|a| a.to_str().unwrap().to_string())
            .collect();
        assert!(args.contains(&"--literal".to_string()));
        assert!(args.contains(&"file.txt".to_string()));
    }

    #[test]
    #[cfg(not(windows))]
    fn register_context_menu_is_err() {
        assert!(register_context_menu().is_err());
    }
}
