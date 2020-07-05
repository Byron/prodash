
Crosstermion is a utility crate to unify some types of both crates, allowing to easily build apps that use the leaner `termion` 
crate on unix systems, but resort to crossterm on windows systems.

Currently provided facilities are:

* a `Key` type an an `input_stream` (_async_) to receive key presses
* an `AltenrativeRawTerminal` which marries an alternative screen with raw mode
* a way to create a `tui` or `tui-react` terminal with either the crossterm or the termion backend.

### Features

All features work additively, but in case they are mutually exclusive, for instance
in case of `tui-react` and `tui`, or `crossterm` and `termion`, the more general one will be chosen.

* _mutually exclusive_
    * **crossterm**
      * provides `Key` conversion support from `crossbeam::event::KeyEvent` and an `AlternativeRawTerminal`
      * provides a threaded key input channel
      * **async**
        * Provides async key input via a futures `Stream`, by spawning a thead
        * **crossterm-async**
           * adds native async capabilites to crossterm, which works without spawning an extra thread thanks to `mio`.
           * also activates `crossterm` dependency
    * **termion**
      * provides `Key` conversion support from `termion::event::Key` and an `AlternativeRawTerminal`
      * provides a threaded key input channel
      * **async**
        * Provides async key input via a futures `Stream`, by spawning a thead
* _mutually exclusive_
    * _using tui_ _(mutually exclusive)_
        * **tui-crossterm** 
          * combines `tui` with `crossterm` and provides a `tui::Terminal` with `crossterm` backend
        * **tui-termion**
          * combines `tui` with `termion` and provides a `tui::Terminal` with `termion` backend
    * _using tui-react_ _(mutually exclusive)_
        * **tui-react-crossterm**
          * combines `tui-react` with `crossterm` and provides a `tui::Terminal` with `crossterm` backend
        * **tui-react-termion**
          * combines `tui-react` with `crossterm` and provides a `tui::Terminal` with `crossterm` backend

