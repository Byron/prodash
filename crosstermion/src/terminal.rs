#[cfg(all(feature = "crossterm", not(feature = "termion")))]
mod _impl {
    use crate::crossterm_utils::into_io_error;
    use crossterm::{
        execute,
        terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use std::io;

    /// Return the horizontal and vertical size of the terminal, if available.
    pub fn size() -> io::Result<(u16, u16)> {
        crossterm::terminal::size().map_err(into_io_error)
    }

    /// A utility writer to activate an alternate screen in raw mode on instantiation, and resets to previous settings on drop.
    ///
    /// Additionally, it will activate _raw_ mode, which causes user input not to show on screen,
    /// and gets handled byte by byte.
    pub struct AlternateRawScreen<T: io::Write> {
        inner: T,
    }

    impl<T: io::Write> AlternateRawScreen<T> {
        pub fn try_from(mut write: T) -> Result<Self, io::Error> {
            terminal::enable_raw_mode().map_err(into_io_error)?;
            execute!(write, EnterAlternateScreen).map_err(crate::crossterm_utils::into_io_error)?;
            Ok(AlternateRawScreen { inner: write })
        }
    }

    impl<T: io::Write> io::Write for AlternateRawScreen<T> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.inner.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.inner.flush()
        }
    }

    impl<T: io::Write> Drop for AlternateRawScreen<T> {
        fn drop(&mut self) {
            terminal::disable_raw_mode().ok();
            execute!(self.inner, LeaveAlternateScreen).ok();
        }
    }

    #[cfg(all(feature = "tui-crossterm-backend", not(feature = "tui-react")))]
    pub mod tui {
        use tui::backend::CrosstermBackend;

        pub fn new_terminal<W: std::io::Write>(write: W) -> Result<tui::Terminal<CrosstermBackend<W>>, std::io::Error> {
            let backend = CrosstermBackend::new(write);
            Ok(tui::Terminal::new(backend)?)
        }
    }

    #[cfg(all(feature = "tui-crossterm-backend", feature = "tui-react"))]
    pub mod tui {
        use tui::backend::CrosstermBackend;

        pub fn new_terminal<W: std::io::Write>(
            write: W,
        ) -> Result<tui_react::Terminal<CrosstermBackend<W>>, std::io::Error> {
            let backend = CrosstermBackend::new(write);
            Ok(tui_react::Terminal::new(backend)?)
        }
    }
}

#[cfg(feature = "termion")]
mod _impl {
    use std::io;
    pub use termion::terminal_size as size;

    pub struct AlternateRawScreen<T: io::Write> {
        inner: termion::screen::AlternateScreen<T>,
    }

    impl<T: io::Write> AlternateRawScreen<termion::raw::RawTerminal<T>> {
        pub fn try_from(write: T) -> Result<Self, io::Error> {
            use termion::raw::IntoRawMode;
            let write = write.into_raw_mode()?;
            Ok(AlternateRawScreen {
                inner: termion::screen::AlternateScreen::from(write),
            })
        }
    }

    impl<T: io::Write> io::Write for AlternateRawScreen<T> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.inner.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.inner.flush()
        }
    }

    #[cfg(all(feature = "tui-termion-backend", not(feature = "tui-react")))]
    pub mod tui {
        use tui::backend::TermionBackend;

        pub fn new_terminal<W: std::io::Write>(write: W) -> Result<tui::Terminal<TermionBackend<W>>, std::io::Error> {
            let backend = TermionBackend::new(write);
            Ok(tui::Terminal::new(backend)?)
        }
    }

    /// Utilities for terminal user interface powered by `tui` or `tui-react`.
    ///
    /// Requires the `tui-react` and `tui-crossterm-backend` features set.
    #[cfg(all(feature = "tui-termion-backend", feature = "tui-react"))]
    pub mod tui {
        use tui::backend::TermionBackend;

        /// Returns a new Terminal instance with a suitable backend.
        pub fn new_terminal<W: std::io::Write>(
            write: W,
        ) -> Result<tui_react::Terminal<TermionBackend<W>>, std::io::Error> {
            let backend = TermionBackend::new(write);
            Ok(tui_react::Terminal::new(backend)?)
        }
    }
}

pub use _impl::*;
