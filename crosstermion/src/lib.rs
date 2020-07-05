/// Features related to terminal user input
pub mod input;
/// Features related to the terminal settings and terminal user interfaces.
///
/// Requires `termion` or `crossterm` feature toggles
#[cfg(any(feature = "termion", feature = "crossterm"))]
pub mod terminal;

#[cfg(feature = "crossterm")]
mod crossterm;
