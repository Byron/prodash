#[cfg(feature = "termion")]
pub mod _impl {
    use crate::tui::input::Key;
    use futures_util::SinkExt;
    use std::{convert::TryInto, io};
    use termion::{
        input::TermRead,
        raw::{IntoRawMode, RawTerminal},
        screen::AlternateScreen,
    };
    use tui::backend::TermionBackend;
    use tui_react::Terminal;

    impl std::convert::TryFrom<termion::event::Key> for Key {
        type Error = termion::event::Key;

        fn try_from(value: termion::event::Key) -> Result<Self, Self::Error> {
            use termion::event::Key::*;
            Ok(match value {
                Backspace => Key::Backspace,
                Left => Key::Left,
                Right => Key::Right,
                Up => Key::Up,
                Down => Key::Down,
                Home => Key::Home,
                End => Key::End,
                PageUp => Key::PageUp,
                PageDown => Key::PageDown,
                BackTab => Key::BackTab,
                Delete => Key::Delete,
                Insert => Key::Insert,
                F(c) => Key::F(c),
                Char(c) => Key::Char(c),
                Alt(c) => Key::Alt(c),
                Ctrl(c) => Key::Ctrl(c),
                Null => Key::Null,
                Esc => Key::Esc,
                _ => return Err(value),
            })
        }
    }

    pub fn new_terminal(
    ) -> Result<Terminal<TermionBackend<AlternateScreen<RawTerminal<io::Stdout>>>>, io::Error> {
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(AlternateScreen::from(stdout));
        Ok(Terminal::new(backend)?)
    }

    pub fn key_input_stream() -> futures_channel::mpsc::Receiver<Key> {
        let (mut key_send, key_receive) = futures_channel::mpsc::channel::<Key>(1);
        // This brings blocking key-handling into the async world
        std::thread::spawn(move || -> Result<(), io::Error> {
            for key in io::stdin().keys() {
                let key: Result<Key, _> = key?.try_into();
                if let Ok(key) = key {
                    smol::block_on(key_send.send(key)).ok();
                }
            }
            Ok(())
        });
        key_receive
    }
}

#[cfg(all(feature = "crossterm", not(feature = "termion")))]
pub mod _impl {
    use crate::tui::input::Key;
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use futures_util::SinkExt;
    use std::{convert::TryInto, io};
    use tui::backend::CrosstermBackend;
    use tui_react::Terminal;

    impl std::convert::TryFrom<crossterm::event::KeyEvent> for Key {
        type Error = crossterm::event::KeyEvent;

        fn try_from(value: crossterm::event::KeyEvent) -> Result<Self, Self::Error> {
            use crossterm::event::{KeyCode::*, KeyModifiers};
            Ok(match value.code {
                Backspace => Key::Backspace,
                Enter => Key::Char('\n'),
                Left => Key::Left,
                Right => Key::Right,
                Up => Key::Up,
                Down => Key::Down,
                Home => Key::Home,
                End => Key::End,
                PageUp => Key::PageUp,
                PageDown => Key::PageDown,
                Tab => Key::Char('\t'),
                BackTab => Key::BackTab,
                Delete => Key::Delete,
                Insert => Key::Insert,
                F(k) => Key::F(k),
                Null => Key::Null,
                Esc => Key::Esc,
                Char(c) => match value.modifiers {
                    KeyModifiers::NONE | KeyModifiers::SHIFT => Key::Char(c),
                    KeyModifiers::CONTROL => Key::Ctrl(c),
                    KeyModifiers::ALT => Key::Alt(c),
                    _ => return Err(value),
                },
            })
        }
    }

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

    pub fn new_terminal(
    ) -> Result<Terminal<CrosstermBackend<AlternateScreen<io::Stdout>>>, io::Error> {
        let backend = CrosstermBackend::new(AlternateScreen::new(io::stdout())?);
        Ok(Terminal::new(backend)?)
    }

    pub fn key_input_stream() -> futures_channel::mpsc::Receiver<Key> {
        let (mut key_send, key_receive) = futures_channel::mpsc::channel::<Key>(1);
        // NOTE: Even though crossterm has support for async event streams, it will use MIO in this
        // case and pull in even more things that we simply don't need for that. A thread and blocking
        // IO will do just fine.
        std::thread::spawn(move || -> Result<(), io::Error> {
            loop {
                let event = crossterm::event::read().map_err(into_io_error)?;
                match event {
                    crossterm::event::Event::Key(key) => {
                        let key: Result<Key, _> = key.try_into();
                        if let Ok(key) = key {
                            smol::block_on(key_send.send(key)).ok();
                        };
                    }
                    _ => continue,
                };
            }
        });
        key_receive
    }
}

#[cfg(not(any(feature = "termion", feature = "crossterm")))]
pub mod _impl {
    use crate::tui::engine::input::Key;
    use std::io;
    use tui::backend::TestBackend;
    use tui_react::Terminal;

    pub fn key_input_stream() -> futures_channel::mpsc::Receiver<Key> {
        compile_error!("use either the 'termion' or the 'crossterm' feature")
    }

    pub fn new_terminal() -> Result<Terminal<TestBackend>, io::Error> {
        Terminal::new(TestBackend::new(100, 100))
    }
}
