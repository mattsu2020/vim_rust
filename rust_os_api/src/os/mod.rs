#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "macos")]
pub use mac::*;

#[cfg(target_os = "haiku")]
mod haiku;
#[cfg(target_os = "haiku")]
pub use haiku::*;

#[cfg(all(unix, not(any(target_os = "macos", target_os = "haiku"))))]
mod unix;
#[cfg(all(unix, not(any(target_os = "macos", target_os = "haiku"))))]
pub use unix::*;
