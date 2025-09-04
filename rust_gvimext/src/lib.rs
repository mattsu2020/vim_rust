#[cfg(windows)]
mod imp {
    use std::cell::Cell;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows::core::{implement, w, GUID, HRESULT, PWSTR};
    use windows::Win32::Foundation::{BOOL, E_FAIL, HKEY, S_OK};
    use windows::Win32::System::Com::{
        IClassFactory, IContextMenu, IDataObject, IShellExtInit, CF_HDROP, FORMATETC, STGMEDIUM,
        TYMED,
    };
    use windows::Win32::System::Registry::{
        RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ,
    };
    use windows::Win32::System::Threading::{CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW};
    use windows::Win32::UI::Shell::{InsertMenuW, CMINVOKECOMMANDINFO, HMENU};
    use windows::Win32::UI::WindowsAndMessaging::DragQueryFileW;

    const BUFSIZE: usize = 1100;
    const EDIT_WITH_VIM_USE_TABPAGES: u32 = 0;
    const EDIT_WITH_VIM_SINGLE: u32 = 1;
    const EDIT_WITH_VIM_DIFF: u32 = 2;

    pub const CLSID_GVIMEXT: GUID = GUID::from_u128(0x51eee242_ad87_11d3_9c1e_0090278bbd99);

    fn get_gvim_name() -> Vec<u16> {
        fn from_registry(root: HKEY) -> Option<Vec<u16>> {
            unsafe {
                let mut key = HKEY::default();
                if RegOpenKeyExW(root, w!("Software\\Vim\\Gvim"), 0, KEY_READ, &mut key).is_ok() {
                    let mut buf = [0u16; BUFSIZE];
                    let mut len = (BUFSIZE * 2) as u32;
                    if RegQueryValueExW(
                        key,
                        w!("path"),
                        None,
                        None,
                        Some(buf.as_mut_slice().align_to_mut::<u8>().1),
                        Some(&mut len),
                    )
                    .is_ok()
                    {
                        let n = (len as usize) / 2;
                        let mut v = buf[..n].to_vec();
                        v.push(0);
                        return Some(v);
                    }
                }
            }
            None
        }
        from_registry(HKEY_CURRENT_USER)
            .or_else(|| from_registry(HKEY_LOCAL_MACHINE))
            .unwrap_or_else(|| {
                let mut v: Vec<u16> = OsStr::new("gvim").encode_wide().collect();
                v.push(0);
                v
            })
    }

    fn get_gvim_invocation(extra: &str) -> Vec<u16> {
        let mut cmd: Vec<u16> = get_gvim_name();
        cmd.pop();
        cmd.extend(OsStr::new(" --literal").encode_wide());
        cmd.extend(OsStr::new(extra).encode_wide());
        cmd.push(0);
        cmd
    }

    #[implement(IShellExtInit, IContextMenu)]
    pub struct ShellExt {
        data: Cell<Option<IDataObject>>,
    }

    impl ShellExt {
        pub fn new() -> Self {
            Self {
                data: Cell::new(None),
            }
        }

        fn file_count(&self) -> u32 {
            if let Some(data) = &self.data.get() {
                unsafe {
                    let fmt = FORMATETC {
                        cfFormat: CF_HDROP.0 as u16,
                        ptd: ptr::null_mut(),
                        dwAspect: 1,
                        lindex: -1,
                        tymed: TYMED::TYMED_HGLOBAL.0 as u32,
                    };
                    let mut stg = STGMEDIUM::default();
                    if data.GetData(&fmt, &mut stg).is_ok() {
                        return DragQueryFileW(stg.Anonymous.hGlobal.0, 0xFFFFFFFF, None);
                    }
                }
            }
            0
        }
    }

    #[allow(non_snake_case)]
    impl IShellExtInit_Impl for ShellExt {
        fn Initialize(
            &self,
            _pidl: *const windows::Win32::UI::Shell::ITEMIDLIST,
            data: Option<&IDataObject>,
            _hkey: HKEY,
        ) -> windows::core::Result<()> {
            self.data.set(data.cloned());
            Ok(())
        }
    }

    #[allow(non_snake_case)]
    impl IContextMenu_Impl for ShellExt {
        fn QueryContextMenu(
            &self,
            hmenu: HMENU,
            index: u32,
            id_first: u32,
            _id_last: u32,
            _flags: u32,
        ) -> windows::core::Result<u32> {
            let count = self.file_count();
            unsafe {
                let text = if count > 1 {
                    "Edit with Vim using tabpages"
                } else {
                    "Edit with Vim"
                };
                let mut wide: Vec<u16> = OsStr::new(text).encode_wide().chain(Some(0)).collect();
                InsertMenuW(hmenu, index, 0, id_first as usize, PWSTR(wide.as_mut_ptr()));
            }
            Ok(1)
        }

        fn InvokeCommand(&self, info: *const CMINVOKECOMMANDINFO) -> windows::core::Result<()> {
            unsafe {
                if (*info).lpVerb as usize > 0xFFFF {
                    return Err(E_FAIL.into());
                }
                let id = (*info).lpVerb as u16 as u32;
                let extra = match id {
                    EDIT_WITH_VIM_USE_TABPAGES => " -p",
                    EDIT_WITH_VIM_DIFF => " -d",
                    _ => "",
                };
                let mut cmd = get_gvim_invocation(extra);
                let mut si = STARTUPINFOW::default();
                si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
                let mut pi = PROCESS_INFORMATION::default();
                CreateProcessW(
                    None,
                    PWSTR(cmd.as_mut_ptr()),
                    None,
                    None,
                    false,
                    0,
                    None,
                    None,
                    &mut si,
                    &mut pi,
                )
                .ok()?;
                Ok(())
            }
        }

        fn GetCommandString(
            &self,
            _id: u32,
            _flags: u32,
            _reserved: *mut u32,
            _buffer: PWSTR,
            _len: u32,
        ) -> windows::core::Result<()> {
            Ok(())
        }
    }

    #[implement(IClassFactory)]
    struct ClassFactory;

    #[allow(non_snake_case)]
    impl IClassFactory_Impl for ClassFactory {
        fn CreateInstance(
            &self,
            _outer: Option<&windows::core::IUnknown>,
            iid: &GUID,
            object: *mut *mut core::ffi::c_void,
        ) -> windows::core::Result<()> {
            let shell: ShellExt = ShellExt::new();
            unsafe {
                shell
                    .cast::<windows::core::IUnknown>()
                    .unwrap()
                    .QueryInterface(iid, object)
                    .ok()?;
            }
            Ok(())
        }
        fn LockServer(&self, _f: BOOL) -> windows::core::Result<()> {
            Ok(())
        }
    }

    #[no_mangle]
    pub extern "system" fn DllCanUnloadNow() -> HRESULT {
        S_OK
    }

    #[no_mangle]
    pub extern "system" fn DllGetClassObject(
        clsid: *const GUID,
        iid: *const GUID,
        out: *mut *mut core::ffi::c_void,
    ) -> HRESULT {
        unsafe {
            if *clsid != CLSID_GVIMEXT {
                return E_FAIL;
            }
            let factory: IClassFactory = ClassFactory.into();
            factory.QueryInterface(iid, out).unwrap_or(E_FAIL)
        }
    }

    #[no_mangle]
    pub extern "system" fn DllRegisterServer() -> HRESULT {
        S_OK
    }

    #[no_mangle]
    pub extern "system" fn DllUnregisterServer() -> HRESULT {
        S_OK
    }
}

#[cfg(not(windows))]
#[no_mangle]
pub extern "system" fn DllRegisterServer() -> i32 {
    1
}

#[cfg(not(windows))]
#[no_mangle]
pub extern "system" fn DllUnregisterServer() -> i32 {
    1
}

#[cfg(not(windows))]
#[no_mangle]
pub extern "system" fn DllGetClassObject(
    _: *const core::ffi::c_void,
    _: *const core::ffi::c_void,
    _: *mut *mut core::ffi::c_void,
) -> i32 {
    1
}

#[cfg(not(windows))]
#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> i32 {
    1
}
