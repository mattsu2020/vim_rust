#[cfg(feature = "tty")]
mod tty;
#[cfg(feature = "tty")]
pub use tty::*;

#[cfg(feature = "tty")]
pub use rust_termlib::*;

#[cfg(feature = "gui")]
mod gui;
#[cfg(feature = "gui")]
pub use gui::*;
