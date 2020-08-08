#![deny(unsafe_code)]

#[cfg(not(feature = "tui-renderer"))]
compile_error!(
    "The `tui-renderer` feature must be set, along with either `tui-renderer-crossterm` or `tui-renderer-termion`"
);
#[cfg(not(any(feature = "tui-renderer-crossterm", feature = "tui-renderer-termion")))]
compile_error!(
    "Please set either the 'tui-renderer-crossterm' or 'tui-renderer-termion' feature whne using the 'tui-renderer'"
);

fn main() -> Result {
    env_logger::init();

    let args: args::Options = argh::from_env();
    Ok(())
}

type Result = std::result::Result<(), Box<dyn Error + Send>>;

mod shared;
use shared::args;

use std::error::Error;
