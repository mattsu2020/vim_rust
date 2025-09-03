#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

#[cfg(not(unix))]
mod fallback;
#[cfg(not(unix))]
pub use fallback::*;
