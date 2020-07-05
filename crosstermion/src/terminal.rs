#[cfg(feature = "crossterm")]
mod _impl {
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
    use std::io;

    pub struct AlternateScreen<T: io::Write> {
        inner: T,
    }

    fn into_io_error(err: crossterm::ErrorKind) -> io::Error {
        if let crossterm::ErrorKind::IoError(err) = err {
            return err;
        }
        unimplemented!("we cannot currently handle non-io errors reported by crossterm")
    }

    impl<T: io::Write> AlternateScreen<T> {
        fn new(mut write: T) -> Result<AlternateScreen<T>, io::Error> {
            enable_raw_mode().map_err(into_io_error)?;
            execute!(write, EnterAlternateScreen).map_err(into_io_error)?;
            Ok(AlternateScreen { inner: write })
        }
    }

    impl<T: io::Write> io::Write for AlternateScreen<T> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.inner.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.inner.flush()
        }
    }

    impl<T: io::Write> Drop for AlternateScreen<T> {
        fn drop(&mut self) {
            disable_raw_mode().ok();
            execute!(self.inner, LeaveAlternateScreen).ok();
        }
    }

    #[cfg(all(feature = "tui", not(feature = "tui-react")))]
    pub mod tui {
        use tui::backend::CrosstermBackend;

        pub fn new_terminal<W: std::io::Write>(
            write: W,
        ) -> Result<tui::Terminal<CrosstermBackend<W>>, std::io::Error> {
            let backend = CrosstermBackend::new(W);
            Ok(tui::Terminal::new(backend)?)
        }
    }

    #[cfg(all(feature = "tui", feature = "tui-react"))]
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

pub use _impl::*;
