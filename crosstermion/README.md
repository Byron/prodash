
Crosstermion is a utility crate to unify some types of both crates, allowing to easily build apps that use the leaner `termion` 
crate on unix systems, but resort to crossterm on windows systems.

Currently provided facilities are:

* a `Key` type an an `input_stream` (_async_) to receive key presses
* an `AltenrativeRawTerminal` which marries an alternative screen with raw mode
* a way to create a `tui` or `tui-react` terminal with either the crossterm or the termion backend.

### But how to do colors and styles?

* **With** `tui`
    * When using the `tui`, you will have native cross-backend support for colors and styles.
* **Without** `tui`
    * Use the **`color`** feature for additional utilities for colors with `ansi_term`.
    * Otherwise, using `ansi_term`, `colored` or `termcolor` will work as expected.
      
### How to build with `crossterm` on Windows and `termion` on Unix?

There seems to be no easy way, as `cargo` will always build dependencies even though they are not supposed to be used on your platform.
This leads to both `termion` and `crossterm` to be built, which is fatal on Windows. Thus one will have to manually select feature toggles
when creating a release build, i.e. one would have to exclude all functionality that requires TUIs by default, and let the user enable
the features they require.

The `compile_error!(…)` macro can be useful to inform users if feature selection is required. Alternatively, assure that everything compiles
even without any selected backend.

Lastly, one can always give in and always compile against `crossterm`.

### Features

All features work additively, but in case they are mutually exclusive, for instance
in case of `tui-react` and `tui`, or `crossterm` and `termion`, the more general one will be chosen.

* _mutually exclusive_
    * **crossterm**
      * provides `Key` conversion support from `crossbeam::event::KeyEvent` and an `AlternativeRawTerminal`
      * provides a threaded key input channel
      * _additive_
        * **input-async-crossterm**
          * adds native async capabilites to crossterm, which works without spawning an extra thread thanks to `mio`.
          * note that threaded key input is always supported.
    * **termion**
      * provides `Key` conversion support from `termion::event::Key` and an `AlternativeRawTerminal`
      * provides a threaded key input channel
      * _additive_
        * **input-async**
          * Spawn a thread and provide input events via a futures Stream
          * note that threaded key input is always supported.
* _mutually exclusive_
    * _using `tui_` _(mutually exclusive)_
        * **tui-termion** _implies `termion` feature_
          * combines `tui` with `termion` and provides a `tui::Terminal` with `termion` backend
        * **tui-crossterm**  _implies `crossterm` feature_
          * combines `tui` with `crossterm` and provides a `tui::Terminal` with `crossterm` backend
    * _using `tui-react`_ _(mutually exclusive)_
        * **tui-react-termion** _implies `termion` feature_
          * combines `tui-react` with `crossterm` and provides a `tui::Terminal` with `crossterm` backend
        * **tui-react-crossterm** _implies `crossterm` feature_
          * combines `tui-react` with `crossterm` and provides a `tui::Terminal` with `crossterm` backend
* **color**
   * Add support for `ansi_term` based conditional coloring. The crate is small, to the point and allows zero-copy drawing
     of bytes and UTF-8 string, while supporting Windows 10 as well.
* _cursor movement_
   * _mutually exclusive_
       * **crossterm**
         * Implements cursor movement with crossterm
       * **termion**
         * Implements cursor movement with termion

