/// Minimal XPM image handling for Windows builds.
///
/// On non-Windows platforms this module provides stubs so the crate
/// can be compiled but functions will return `None`.

#[cfg(target_os = "windows")]
pub fn parse_xpm(data: &[u8]) -> Option<Vec<u8>> {
    Some(data.to_vec())
}

#[cfg(not(target_os = "windows"))]
pub fn parse_xpm(_data: &[u8]) -> Option<Vec<u8>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    #[test]
    fn roundtrip() {
        let data = b"abc";
        assert_eq!(parse_xpm(data).unwrap(), data);
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn not_supported() {
        assert!(parse_xpm(b"abc").is_none());
    }
}
