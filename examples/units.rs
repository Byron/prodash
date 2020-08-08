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
    let root = Tree::default();
    let renderer = args.renderer.clone().unwrap_or("line".into());
    let handle = shared::launch_ambient_gui(root.clone(), &renderer, args).unwrap();
    let work = async move {
        let mut unblock = blocking::Unblock::new(());
        unblock.with_mut(move |_| work_for_a_long_time_blocking(root)).await
    };
    futures_lite::future::block_on(futures_util::future::select(handle, work.boxed()));
    Ok(())
}

fn work_for_a_long_time_blocking(root: Tree) {
    let mut bytes_max = root.add_child("download");
    bytes_max.init(
        Some(100_000_000),
        Some(unit::dynamic_and_mode(unit::Bytes, unit::Mode::PercentageAfterUnit)),
    );
    loop {
        bytes_max.inc_by(1_459_121);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

type Result = std::result::Result<(), Box<dyn Error + Send + 'static>>;

mod shared;
use shared::args;

use futures_lite::FutureExt;
use prodash::{unit, Tree};
use std::error::Error;
