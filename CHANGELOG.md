## v9.0.0

### Breaking

* add `set_name(…)` and `name()` to `Progress` trait.

## v8.0.1 - Add missing trailing paranthesis in throughput display

## v8.0.0

### New Features

* Provide various Units for improved formatting of durations, steps, counts and bytes, and specify display of percentages.
* Support for throughput display if enabled in _line_ and _tui_ renderer, and when opted in when creating units.
* Turn `ProgressStep` type into usize to allow for a broader range of values. Specifically, this enables counting bytes read and written
  of files or streams bigger than 2^32-1
* A new example program: **units**
* Add methods to help determine key hierarchy, useful for writing custom renderers:
  * `Key::shares_parent_with(…)`
  * `Key::adjecency(…)`
* Add `Progress` trait to allow make progress generic
* Add `progress::Discard` and `progress::DoOrDiscard` to make optional progress more convenient, especially when using the `Progress` trait.
* Add `progress-tree` feature to gate the `dashmap` powered tree (enabled by default)

### Breaking

* In `Progress::init(max, unit)` , `unit` now is `Option<Unit>`, existing code can be transformed using `progress.init(None, Some("label".into()))`.
* moved`tree::progress` into `progress` and renamed `Value` into `Task` and `Progress` into `Value`
* moved`tree::messages` into `crates::messages`
* moved`tree::key` into `crates::progress::key`
* moved `tree::Throughput` into `crate::Throughput`
* **removed** `deep_eq()` method in `Root` tree
* tui engine option `redraw_only_on_state_change` was removed without substitute
* **Move and rename**
  * `tree::ProgressState` → `progress::State`
  * `tree::Value` → `progress::Value`
  * `tree::Progress` → `Progress`
* Remove `Hash` implementation for all public types except for `tree::Key`
* Move `tui` and `line` renderers into the `render` module
* Rename `log-renderer` feature to `progress-tree-log`
* Rename `tui-renderer*` into `render-tui*` and `line-renderer*` into `render-line*`

## v7.1.1

* dependency update: smol 0.2

## v7.1.0 

* Improved looks thanks to bold fonts

## v7.0.4 

* **tree::Item**
  * Add new methods `inc_by(step)` and `inc()` for convenience 
* **line renderer**
  * They now look clearer, as they changed from \[===>     ] to \[===>------]

## v7.0.3 - cleanup and code simplification in the line renderer

## v7.0.2

* **render-line** `JoinHandle` will 
  * now send a signal to perform a render before shutting down to capture the final state
  * wait for the render thread to complete the aforementioned actions on drop. You can still override this behaviour through
   `disconnect()` or `forget()`.
  * removed special code-paths that avoided bringing up another thread for 'ticks' at the expense of shutdown delay.

## v7.0.1 - Prevent cursor movement if no progress bar is drawn

## v7.0.0 - Add new render-line, change feature flag names

### New Features

* **line**
  * There is a new line renderer as neat trade off between bare logs and full-blown tui. It will work best with 'simple' and not
    too dynamically changing progress trees.
  * Activate it with the `render-line` + one of `render-line-crossterm` or `render-line-termion` feature flags.
* **tree**
  * New methods in **Root*
    * `copy_new_messages(…)` method, copying only messages seen since last time it was called
    * `message_buffer_usage(…)` returns the amount of message we currently store, as well as the ring buffer capacity

### Breaking Changes

* **`tui` module**
    * **TuiOptions** -> **Options**
    * `render_with_input` now takes the Write stream as argument, instead of defaulting to `std::io::stdout()`
* **Feature Flags**
    * **with-crossterm** -> **render-tui-crossterm**
    * **with-termion** -> **render-tui-termion**

## v6.0.0 - Factor terminal input into the new `crosstermion` crate

Due to this work, the default features changed, which is a breaking change for those who relied on it.
Now when using the `render-tui`, one will also have to specify either the `with-crossbeam` or `render-tui-termion` feature.

## v5.0.0 - Support for windows by using Crossbeam by default

A first low-effort move to the latest version should be to set the dependency to
`default-features = false, features = ["render-tui", "render-tui-termion", "localtime", "log-renderer"]`
to get the same configuration as before.

To try crossbeam, use `with-crossbeam` instead of `render-tui-termion`.

If you have been using the event stream to send your own keys, swap `termion::event::Key` with `prodash::tui::input::Key`.

## v4.1.0 - Allow the TUI to automatically stop if there is no progress to display

This way, it's easier to use `prodash::tui` for visualizing finite tasks, which originally it wasn't intended for.

Previously, in order to achieve the same, one would have to initialize the TUI with an event stream and send the Event
for shutting down once the task at hand is complete.

## v4.0.5 - Fix delayed reset of the terminal

Previously even after the future was dropped, it seemed like the terminal wasn't reset and the user was required
to explicitly flush stdout to make the changes appear. This is due to the flushing previously happening too early,
that is, before the `terminal` was dropped which emits the respective terminal escape codes at this time.

Now the terminal instance is dropped explicitly right before emitting a flush.
One might argue that the flush should happen in the terminal instance itself, but fixing that is out of scope.

## v4.0.4 - Simplify `message()` trait bounds

## v4.0.3 - Remove piper in favor of futures-channel (which was included anyway)

## v4.0.2 - Upgrade to latest TUI and TUI-react crates

## v4.0.1 - Reduce theoretical direct dependencies by not using 'futures' crate directly

## v4.0.0 - Switch from futures executor to smol

This actually simplifies some parts of the implementation, while fixing issues along futures not being dropped while they
were on a thread pool. Now, for the example, no threadpool is used anymore.

**Note** that this also means that in order for each frame to be drawn, one would have to invoke `smol::run` in one thread to
activate the reactor which processes the timeout/ticker. Alternatively, one would send `Tick` events through a channel to trigger
a redraw manually.

## v3.6.3 - Fix out-of-bounds access (and panic) due to new and more precise progress bars

## v3.6.2 - More horizontally precise progress bars; progress bars are now have lines between them vertically

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
