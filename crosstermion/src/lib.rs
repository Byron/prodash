/// Features related to terminal user input
pub mod input;
/// Features related to the terminal settings and terminal user interfaces.
///
/// Requires `termion` or `crossterm` feature toggles
#[cfg(any(feature = "termion", feature = "crossterm"))]
pub mod terminal;

/// Return true if we should colorize the output, based on [clicolors spec](https://bixense.com/clicolors/) and [no-color spec](https://no-color.org)
///
/// Note that you should also validate that the output stream is actually connected to a terminal, which usually looks like
/// `atty::is(atty::Stream::Stdout) && should_colorize()
pub fn should_colorize() -> bool {
    allow_clicolors_spec() && allow_by_no_color_spec()
}

fn evar_with_default<'a>(name: &str, default: &'a str) -> Cow<'a, OsStr> {
    std::env::var_os(name)
        .map(Cow::from)
        .unwrap_or_else(|| Cow::Borrowed(OsStr::new(default)))
}

fn evar_equals(var: Cow<OsStr>, want: &str) -> bool {
    var == Cow::Borrowed(OsStr::new(want))
}

fn evar_not_equals(var: Cow<OsStr>, want: &str) -> bool {
    var != Cow::Borrowed(OsStr::new(want))
}

// https://bixense.com/clicolors/
fn allow_clicolors_spec() -> bool {
    evar_equals(evar_with_default("CLICOLOR", "1"), "1")
        || evar_not_equals(evar_with_default("CLICOLOR_FORCE", "0"), "0")
}

// https://no-color.org
fn allow_by_no_color_spec() -> bool {
    std::env::var_os("NO_COLOR").is_none()
}

#[cfg(feature = "crossterm")]
mod crossterm_utils;

// Reexports
#[cfg(feature = "crossbeam-channel")]
pub use crossbeam_channel;
#[cfg(feature = "crossterm")]
pub use crossterm;
#[cfg(feature = "flume")]
pub use flume;
use std::borrow::Cow;
use std::ffi::OsStr;
#[cfg(feature = "termion")]
pub use termion;
#[cfg(feature = "tui")]
pub use tui;
#[cfg(feature = "tui-react")]
pub use tui_react;
