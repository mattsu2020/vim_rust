use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct NbMessage {
    pub command: String,
}

pub async fn echo_nb(msg: NbMessage) -> NbMessage {
    msg
}

#[no_mangle]
pub extern "C" fn nb_serialize(cmd: *const c_char) -> *mut c_char {
    let cstr = unsafe { CStr::from_ptr(cmd) };
    let msg = NbMessage {
        command: cstr.to_str().unwrap().to_string(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn nb_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { drop(CString::from_raw(ptr)); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn async_echo() {
        let msg = NbMessage { command: "ping".into() };
        let reply = echo_nb(msg.clone()).await;
        assert_eq!(reply, msg);
    }

    #[test]
    fn serde_roundtrip() {
        let msg = NbMessage { command: "test".into() };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: NbMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
    }
}
