#![deny(unsafe_code)]
/*!
Prodash is a dashboard for displaying the progress of concurrent application.

It consists of two parts

* a `Tree` to gather progress information and messages
* a terminal user interface which displays this information, along with optional free-form information provided by the application itself

Even though the `Tree` is not async, it's meant to be transparent and non-blocking performance wise, and benchmarks seem to indicate this
is indeed the case.

The **terminal user interface** seems to be the least transparent part, but can be configured to refresh less frequently.

# Terminal User Interface

By default, a TUI is provided to visualize all state. Have a look at [the example provided in the tui module](./tui/index.html).

# Logging

If the feature `log-renderer` is enabled (default), most calls to `progress` will also be logged.
That way, even without a terminal user interface, there will be progress messages.
Please note that logging to stdout should not be performed with this feature enabled and a terminal user interface, as this will
seriously interfere with the TUI.

# A demo application

Please have a look at the [dashboard demo](https://github.com/Byron/crates-io-cli-rs/blob/master/prodash/examples/dashboard.rs).

[![asciicast](https://asciinema.org/a/301838.svg)](https://asciinema.org/a/301838)

Run it with `cargo run --example dashboard` and see what else it can do by checking out `cargo run --example dashboard -- --help`.

# Changelog

## v1.1.0

* fix toggles - previously prodash, withoug tui, would always build humantime and unicode width
* add support for logging as user interface

*/
mod config;
pub mod tree;

pub use config::TreeOptions;
pub use tree::Root as Tree;

#[cfg(feature = "tui-renderer")]
pub mod tui;

#[cfg(feature = "log-renderer")]
pub use log::info;
#[cfg(feature = "log-renderer")]
pub use log::warn;

#[cfg(not(feature = "log-renderer"))]
mod log {
    /// Stub
    #[macro_export(local_inner_macros)]
    macro_rules! warn {
        (target: $target:expr, $($arg:tt)+) => {};
        ($($arg:tt)+) => {};
    }
    /// Stub
    #[macro_export(local_inner_macros)]
    macro_rules! info {
        (target: $target:expr, $($arg:tt)+) => {};
        ($($arg:tt)+) => {};
    }
}
