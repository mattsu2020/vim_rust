#![cfg_attr(not(target_os = "windows"), allow(unused))]

#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::iter::once;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
#[cfg(target_os = "windows")]
use std::path::Path;

#[cfg(target_os = "windows")]
use windows::core::{PCWSTR, PWSTR};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{CloseHandle, HWND};
#[cfg(target_os = "windows")]
use windows::Win32::System::Registry::{
    RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK};

#[cfg(target_os = "windows")]
const BUFSIZE: usize = 260;

#[cfg(target_os = "windows")]
fn get_gvim_name() -> String {
    unsafe fn query(root: HKEY) -> Option<String> {
        let subkey: Vec<u16> = OsStr::new("Software\\Vim\\Gvim")
            .encode_wide()
            .chain(once(0))
            .collect();
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(root, PCWSTR(subkey.as_ptr()), 0, KEY_READ, &mut hkey).is_ok() {
            let value: Vec<u16> = OsStr::new("path").encode_wide().chain(once(0)).collect();
            let mut buf = [0u16; BUFSIZE];
            let mut len = (buf.len() * 2) as u32;
            if RegQueryValueExW(
                hkey,
                PCWSTR(value.as_ptr()),
                None,
                None,
                Some(buf.as_mut_ptr() as *mut u8),
                Some(&mut len),
            )
            .is_ok()
            {
                let s = String::from_utf16_lossy(&buf[..(len as usize / 2)]);
                return Some(s.trim_end_matches('\0').to_string());
            }
        }
        None
    }
    query(HKEY_CURRENT_USER)
        .or_else(|| query(HKEY_LOCAL_MACHINE))
        .unwrap_or_else(|| "gvim.exe".to_string())
}

#[cfg(target_os = "windows")]
fn get_gvim_invocation() -> String {
    let mut name = get_gvim_name();
    name.push_str(" --literal");
    name
}

#[cfg(target_os = "windows")]
fn to_wide<S: AsRef<OsStr>>(s: S) -> Vec<u16> {
    s.as_ref().encode_wide().chain(once(0)).collect()
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
pub enum GvimExtraOptions {
    NoOptions,
    InDiffMode,
    UseTabpages,
}

#[cfg(target_os = "windows")]
pub fn invoke_single_gvim(
    _parent: HWND,
    working_dir: &Path,
    files: &[&Path],
    extra: GvimExtraOptions,
) {
    let mut cmd = get_gvim_invocation();
    match extra {
        GvimExtraOptions::InDiffMode => cmd.push_str(" -d"),
        GvimExtraOptions::UseTabpages => cmd.push_str(" -p"),
        GvimExtraOptions::NoOptions => {}
    }
    for f in files {
        cmd.push(' ');
        cmd.push('"');
        cmd.push_str(&f.display().to_string());
        cmd.push('"');
    }
    let mut cmd_w = to_wide(cmd);
    let dir_w = to_wide(working_dir);
    unsafe {
        let mut si = STARTUPINFOW::default();
        si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        let mut pi = PROCESS_INFORMATION::default();
        let success = CreateProcessW(
            PCWSTR::null(),
            PWSTR(cmd_w.as_mut_ptr()),
            None,
            None,
            false,
            0,
            None,
            PCWSTR(dir_w.as_ptr()),
            &mut si,
            &mut pi,
        )
        .as_bool();
        if success {
            CloseHandle(pi.hProcess);
            CloseHandle(pi.hThread);
        } else {
            let msg = to_wide("Error creating process: Check if gvim is in your path!");
            let title = to_wide("gvimext.dll error");
            MessageBoxW(_parent, PCWSTR(msg.as_ptr()), PCWSTR(title.as_ptr()), MB_OK);
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn invoke_single_gvim() {
    panic!("rust_gvimext only works on Windows");
}
