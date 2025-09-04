use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;

use windows::core::{implement, Result, PWSTR};
use windows::Win32::Foundation::HKEY;
use windows::Win32::System::Com::IDataObject;
use windows::Win32::System::Registry::{RegSetKeyValueW, HKEY_CLASSES_ROOT, REG_SZ};
use windows::Win32::UI::Shell::{IContextMenu, IShellExtInit, CMINVOKECOMMANDINFO};
use windows::Win32::UI::WindowsAndMessaging::HMENU;

/// COM implementation providing the context menu entry.
#[implement(IShellExtInit, IContextMenu)]
pub struct VimShellExt;

#[allow(non_snake_case)]
impl VimShellExt {
    /// Called by the Shell to initialize the extension.
    pub fn Initialize(
        &self,
        _pidl: *mut std::ffi::c_void,
        _data: Option<&IDataObject>,
        _hkey: HKEY,
    ) -> Result<()> {
        Ok(())
    }

    /// Adds items to the context menu; the implementation simply
    /// reports that no items were added.  A full port would insert the
    /// "Edit with Vim" entry here.
    pub fn QueryContextMenu(
        &self,
        _menu: HMENU,
        _index_menu: u32,
        _id_cmd_first: u32,
        _id_cmd_last: u32,
        _flags: u32,
    ) -> Result<u32> {
        Ok(0)
    }

    /// Invokes the selected command.  This stub does not perform any
    /// action beyond returning success.
    pub fn InvokeCommand(&self, _info: *const CMINVOKECOMMANDINFO) -> Result<()> {
        Ok(())
    }

    /// Retrieves help text for a command.  Not used in this stub.
    pub fn GetCommandString(
        &self,
        _id_cmd: u32,
        _u_type: u32,
        _reserved: u32,
        _command: PWSTR,
        _cch: u32,
    ) -> Result<()> {
        Ok(())
    }
}

/// Register the "Edit with Vim" context menu entry in the Windows registry.
pub fn register_context_menu() -> std::io::Result<()> {
    let key = super::context_menu_key();
    let value = "\"gvim.exe\" \"%1\"";
    let mut key_w: Vec<u16> = OsStr::new(key).encode_wide().chain(Some(0)).collect();
    let value_w: Vec<u16> = OsStr::new(value).encode_wide().chain(Some(0)).collect();
    unsafe {
        RegSetKeyValueW(
            HKEY_CLASSES_ROOT,
            PWSTR(key_w.as_mut_ptr()),
            PWSTR::null(),
            REG_SZ,
            Some(value_w.as_ptr() as *const u8),
            (value_w.len() * 2) as u32,
        )
        .ok()?;
    }
    Ok(())
}

/// Launch `gvim.exe` with the provided files.
pub fn open_files<I, P>(files: I) -> std::io::Result<()>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut cmd = Command::new("gvim.exe");
    for f in files {
        cmd.arg(f.as_ref());
    }
    cmd.spawn().map(|_| ())
}
