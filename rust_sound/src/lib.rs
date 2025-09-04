use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::fs::File;
use std::io::BufReader;
use std::os::raw::{c_char, c_int, c_long};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

struct SoundHandle {
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    sink: Sink,
}

struct Manager {
    next_id: c_long,
    sounds: HashMap<c_long, SoundHandle>,
}

impl Manager {
    fn new() -> Self {
        Self { next_id: 1, sounds: HashMap::new() }
    }

    fn play_file(&mut self, path: &CStr) -> c_long {
        if let Ok(p) = path.to_str() {
            if let Ok((stream, handle)) = OutputStream::try_default() {
                if let Ok(sink) = Sink::try_new(&handle) {
                    if let Ok(file) = File::open(p) {
                        if let Ok(source) = Decoder::new(BufReader::new(file)) {
                            sink.append(source);
                            let id = self.next_id;
                            self.next_id += 1;
                            self.sounds.insert(id, SoundHandle { _stream: stream, _handle: handle, sink });
                            return id;
                        }
                    }
                }
            }
        }
        0
    }

    fn stop(&mut self, id: c_long) {
        if let Some(h) = self.sounds.remove(&id) {
            h.sink.stop();
        }
    }

    fn clear(&mut self) {
        for (_, h) in self.sounds.drain() {
            h.sink.stop();
        }
    }
}

thread_local! {
    static MANAGER: RefCell<Manager> = RefCell::new(Manager::new());
}

#[no_mangle]
pub extern "C" fn rs_sound_playfile(path: *const c_char) -> c_long {
    if path.is_null() {
        return 0;
    }
    let cstr = unsafe { CStr::from_ptr(path) };
    MANAGER.with(|m| m.borrow_mut().play_file(cstr))
}

#[no_mangle]
pub extern "C" fn rs_sound_playevent(name: *const c_char) -> c_long {
    rs_sound_playfile(name)
}

#[no_mangle]
pub extern "C" fn rs_sound_stop(id: c_long) {
    MANAGER.with(|m| m.borrow_mut().stop(id));
}

#[no_mangle]
pub extern "C" fn rs_sound_clear() {
    MANAGER.with(|m| m.borrow_mut().clear());
}

#[no_mangle]
pub extern "C" fn rs_has_any_sound_callback() -> c_int {
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn play_missing_file() {
        let p = CString::new("/no/such/file.wav").unwrap();
        assert_eq!(rs_sound_playfile(p.as_ptr()), 0);
    }
}
