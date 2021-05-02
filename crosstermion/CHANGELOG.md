#### v0.7.0

* upgrade to TUI 0.15

#### v0.4.0

* upgrade to TUI 0.12

#### v0.3.2

* Remove `futures-util` dependency in favor of `futures-lite`

#### v0.2.0

* remove native support for `flume` and `crossbeam-channel` for key input in favor of `std::sync::mpsc::Receiver`s :).

#### v0.1.5

* support for simple cursor movements
* get terminal size
* color support to learn if color is allowed, and dynamically disable it
  if `ansi_term` is used.

#### v0.1.4

* Fix precendence of imports in 'terminal' module.

#### v0.1.3

* `Key` type conversions are now always compiled when possible, as they are not mutually exclusive

#### v0.1.2

* Enable `flume/select` by default and allow selecting `flume/async` via the `flume-async` feature.

#### v0.1.1

* Add support for 'input-thread-flume' for using flume channels instead of crossbeam ones. They are
  smaller and there is no unsafe code either, at the expense of lack of the 'select!()` capability.

#### v0.1.0

Initial release.
