use std::collections::HashMap;
use std::ffi::CStr;
use std::fs::read_to_string;
use std::os::raw::{c_char, c_int};

#[no_mangle]
pub extern "C" fn xpm_load(path: *const c_char, width: *mut c_int, height: *mut c_int) -> *mut u32 {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let c_path = unsafe { CStr::from_ptr(path) };
    let path_str = match c_path.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let content = match read_to_string(path_str) {
        Ok(c) => c,
        Err(_) => return std::ptr::null_mut(),
    };
    // Extract lines between quotes
    let mut lines: Vec<String> = Vec::new();
    for line in content.lines() {
        if let Some(start) = line.find('"') {
            if let Some(end) = line[start + 1..].find('"') {
                lines.push(line[start + 1..start + 1 + end].to_string());
            }
        }
    }
    if lines.is_empty() {
        return std::ptr::null_mut();
    }
    let mut parts = lines[0].split_whitespace();
    let w: usize = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let h: usize = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let colors: usize = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let cpp: usize = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    if w == 0 || h == 0 || cpp == 0 {
        return std::ptr::null_mut();
    }
    let mut map: HashMap<String, u32> = HashMap::new();
    for i in 1..=colors {
        if let Some(line) = lines.get(i) {
            if line.len() < cpp {
                continue;
            }
            let key = &line[..cpp];
            if let Some(pos) = line[cpp..].find("c ") {
                let col = &line[cpp + pos + 2..];
                if col.trim() == "None" {
                    map.insert(key.to_string(), 0);
                } else if col.starts_with('#') && col.len() >= 7 {
                    let r = u8::from_str_radix(&col[1..3], 16).unwrap_or(0) as u32;
                    let g = u8::from_str_radix(&col[3..5], 16).unwrap_or(0) as u32;
                    let b = u8::from_str_radix(&col[5..7], 16).unwrap_or(0) as u32;
                    let val = 0xFF000000 | (r << 16) | (g << 8) | b;
                    map.insert(key.to_string(), val);
                }
            }
        }
    }
    let mut pixels: Vec<u32> = Vec::with_capacity(w * h);
    for y in 0..h {
        if let Some(line) = lines.get(1 + colors + y) {
            for x in 0..w {
                let idx = x * cpp;
                if idx + cpp <= line.len() {
                    let key = &line[idx..idx + cpp];
                    let color = map.get(key).copied().unwrap_or(0);
                    pixels.push(color);
                } else {
                    pixels.push(0);
                }
            }
        }
    }
    unsafe {
        if !width.is_null() {
            *width = w as c_int;
        }
        if !height.is_null() {
            *height = h as c_int;
        }
    }
    let mut pixels = pixels.into_boxed_slice();
    let ptr = pixels.as_mut_ptr();
    std::mem::forget(pixels);
    ptr
}

#[no_mangle]
pub extern "C" fn xpm_invert(data: *mut u32, len: usize) {
    if data.is_null() || len == 0 {
        return;
    }
    let slice = unsafe { std::slice::from_raw_parts_mut(data, len) };
    for pixel in slice {
        let a = *pixel & 0xFF000000;
        let rgb = *pixel & 0x00FFFFFF;
        *pixel = a | (!rgb & 0x00FFFFFF);
    }
}

#[no_mangle]
pub extern "C" fn xpm_free(data: *mut u32, len: usize) {
    if data.is_null() {
        return;
    }
    unsafe {
        Vec::from_raw_parts(data, len, len);
    }
}
