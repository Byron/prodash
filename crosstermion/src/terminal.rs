#[cfg(feature = "crossterm")]
mod _impl {
    use crossterm::{
        execute,
        terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use std::io;

    pub struct AlternateRawScreen<T: io::Write> {
        inner: T,
    }

    impl<T: io::Write> AlternateRawScreen<T> {
        pub fn try_from(mut write: T) -> Result<Self, io::Error> {
            terminal::enable_raw_mode().map_err(crate::crossterm::into_io_error)?;
            execute!(write, EnterAlternateScreen).map_err(crate::crossterm::into_io_error)?;
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

    #[cfg(all(feature = "tui/crossterm", not(feature = "tui-react")))]
    pub mod tui {
        use tui::backend::CrosstermBackend;

        pub fn new_terminal<W: std::io::Write>(
            write: W,
        ) -> Result<tui::Terminal<CrosstermBackend<W>>, std::io::Error> {
            let backend = CrosstermBackend::new(W);
            Ok(tui::Terminal::new(backend)?)
        }
    }

    #[cfg(all(feature = "tui/crossterm", feature = "tui-react"))]
    pub mod tui {
        use tui::backend::CrosstermBackend;

        pub fn new_terminal<W: std::io::Write>(
            write: W,
        ) -> Result<tui_react::Terminal<CrosstermBackend<W>>, std::io::Error> {
            let backend = CrosstermBackend::new(W);
            Ok(tui_react::Terminal::new(backend)?)
        }
    }
}

#[cfg(all(feature = "termion", not(feature = "crossterm")))]
mod _impl {
    use std::io;
    pub use termion::screen::AlternateScreen;

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

    #[cfg(all(feature = "tui/termion", not(feature = "tui-react")))]
    pub mod tui {
        use tui::backend::TermionBackend;

        pub fn new_terminal<W: std::io::Write>(
        ) -> Result<tui::Terminal<TermionBackend<W>>, std::io::Error> {
            let backend = TermionBackend::new(W);
            Ok(tui::Terminal::new(backend)?)
        }
    }

    #[cfg(all(feature = "tui/termion", feature = "tui-react"))]
    pub mod tui {
        use tui::backend::TermionBackend;

        pub fn new_terminal<W: std::io::Write>(
            write: W,
        ) -> Result<tui_react::Terminal<TermionBackend<W>>, std::io::Error> {
            let backend = TermionBackend::new(write);
            Ok(tui_react::Terminal::new(backend)?)
        }
    }
}

pub use _impl::*;
