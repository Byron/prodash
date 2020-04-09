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

## v3.6.1 - Properly respond to state changes even when 'redraw_only_on_state_change' is enabled

## v3.6.0 - A TUI option to only redraw if the progress actually changed. Useful if the change rate is lower than the frames per second.

## v3.5.1 - Don't copy messages if the message pane is hidden, saving time

## v3.5.0 - Cleaner visuals for hierarchical progress items, these won't show lines if there are no direct children with progress

## v3.4.1 - Enable localtime support by default

## v3.4.0 - Even nicer tree rendering, along with screen space savings

## v3.3.0 - Much nicer task tree visualization

## v3.2.0 - Application can control if the GUI will respond to interrupt requests

## v3.1.1 - Bugfix (really): Finally delayed column resizing works correctly.

## v3.1.0 - Tree::halted(…) indicates interruptable tasks without progress. Tree::blocked(…) means non-interruptable without progress.

## v3.0.2 - Bugfix: Allow column-width computation to recover from becoming 0

## v3.0.1 - Bugfix: Don't allow values of 0 for when to recompute task column widths

## v3.0.0 - New TUI option to delay computation of column width for stability with rapidly changing tasks

## v2.1.0 - Optional cargo feature "localtime" shows all times in the local timezone

## v2.0.1 - fix integer underflow with graphemes that report width of 0

## v2.0.0

* BREAKING: `progress.blocked(eta)` now takes a statically known reason for the blocked state `progress.blocked(reason, eta)`. This is
  useful to provide more context.

## v1.2.0

* Support for eta messages in blocked unbounded tasks

## v1.1.6

* improve API symmetry by providing a `Tree::name()` to accompany `Tree::set_name(…)`

## v1.1.5

* Flush stdout when the TUI stopped running. That way, the alternate/original screen will be shown right away.

## v1.1.4

* Don't pretend to use &str if in fact an owned string is required. This caused unnecessary clones for those who pass owned strings.

## v1.1.3

* hide cursor or a nicer visual experience

## v1.1.0

* fix toggles - previously prodash, withoug tui, would always build humantime and unicode width
* add support for logging as user interface

*/
mod config;
pub mod tree;

pub use config::TreeOptions;
pub use tree::Root as Tree;

#[cfg(test)]
mod tree_tests;

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
