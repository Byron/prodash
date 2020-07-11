/// Features related to terminal user input
pub mod input;
/// Features related to the terminal settings and terminal user interfaces.
///
/// Requires `termion` or `crossterm` feature toggles
#[cfg(any(feature = "termion", feature = "crossterm"))]
pub mod terminal;

#[cfg(any(feature = "termion", feature = "crossterm"))]
pub mod cursor;

pub mod color;

#[cfg(feature = "crossterm")]
pub mod crossterm_utils;

// Reexports
#[cfg(feature = "ansi_term")]
pub use ansi_term;
#[cfg(feature = "crossterm")]
pub use crossterm;
#[cfg(feature = "termion")]
pub use termion;
#[cfg(feature = "tui")]
pub use tui;
#[cfg(feature = "tui-react")]
pub use tui_react;
