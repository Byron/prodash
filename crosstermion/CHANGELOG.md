#### v0.1.3

* `Key` type conversions are now always compiled when possible, as they are not mutually exclusive

#### v0.1.2

* Enable `flume/select` by default and allow selecting `flume/async` via the `flume-async` feature.

#### v0.1.1

* Add support for 'input-thread-flume' for using flume channels instead of crossbeam ones. They are
  smaller and there is no unsafe code either, at the expense of lack of the 'select!()` capability.

#### v0.1.0

Initial release.
