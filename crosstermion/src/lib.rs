pub mod input;
#[cfg(any(feature = "termion", feature = "crossterm"))]
pub mod terminal;

#[cfg(feature = "crossterm")]
mod crossterm;
