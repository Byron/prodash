/// A set of possible key presses, equivalent to the one in `termion@1.5.5::event::Key`
#[derive(Debug, Clone, Copy)]
pub enum Key {
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Alt(char),
    Ctrl(char),
    Null,
    Esc,
}

#[cfg(any(feature = "crossterm", feature = "termion"))]
enum Action<T> {
    Continue,
    Result(Result<T, std::io::Error>),
}

#[cfg(any(feature = "crossterm", feature = "termion"))]
fn continue_on_interrupt<T>(result: Result<T, std::io::Error>) -> Action<T> {
    match result {
        Ok(v) => Action::Result(Ok(v)),
        Err(err) if err.kind() == std::io::ErrorKind::Interrupted => Action::Continue,
        Err(err) => Action::Result(Err(err)),
    }
}

mod convert {
    #[cfg(any(feature = "crossterm", feature = "termion"))]
    use super::Key;

    #[cfg(feature = "crossterm")]
    /// Convert from `crossterm::event::KeyEvent`
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

    #[cfg(feature = "termion")]
    /// Convert from `termion::event::Key`
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
}

#[cfg(feature = "crossterm")]
mod _impl {
    use crate::input::{continue_on_interrupt, Action};

    /// Return a receiver of user input events to avoid blocking the main thread.
    pub fn key_input_channel() -> std::sync::mpsc::Receiver<super::Key> {
        use std::convert::TryInto;

        let (key_send, key_receive) = std::sync::mpsc::sync_channel(0);
        std::thread::spawn(move || -> Result<(), std::io::Error> {
            loop {
                let event = match continue_on_interrupt(
                    crossterm::event::read().map_err(crate::crossterm_utils::into_io_error),
                ) {
                    Action::Continue => continue,
                    Action::Result(res) => res?,
                };
                match event {
                    crossterm::event::Event::Key(key) => {
                        let key: Result<super::Key, _> = key.try_into();
                        if let Ok(key) = key {
                            if key_send.send(key).is_err() {
                                break;
                            }
                        };
                    }
                    _ => continue,
                };
            }
            Ok(())
        });
        key_receive
    }

    /// Return a stream of key input Events
    ///
    /// Requires the `input-async` feature.
    #[cfg(feature = "input-async-crossterm")]
    pub fn key_input_stream() -> impl futures_core::stream::Stream<Item = super::Key> {
        use futures_lite::StreamExt;
        use std::convert::TryFrom;
        crossterm::event::EventStream::new()
            .filter_map(|r| r.ok())
            .filter_map(|e| match e {
                crossterm::event::Event::Key(key) => super::Key::try_from(key).ok(),
                _ => None,
            })
    }
}

#[cfg(all(feature = "termion", not(feature = "crossterm")))]
mod _impl {
    use crate::input::{continue_on_interrupt, Action};

    /// Return a stream of user input events.
    ///
    /// Requires feature `futures-channel`
    #[cfg(feature = "input-async")]
    pub fn key_input_stream() -> impl futures_core::stream::Stream<Item = super::Key> {
        use std::{convert::TryInto, io};
        use termion::input::TermRead;

        let (key_send, key_receive) = async_channel::bounded::<super::Key>(1);
        // This brings blocking key-handling into the async world
        std::thread::spawn(move || -> Result<(), io::Error> {
            for key in io::stdin().keys() {
                let key: Result<super::Key, _> = match continue_on_interrupt(key) {
                    Action::Continue => continue,
                    Action::Result(res) => res?.try_into(),
                };
                if let Ok(key) = key {
                    if futures_lite::future::block_on(key_send.send(key)).is_err() {
                        break;
                    }
                }
            }
            Ok(())
        });
        key_receive
    }

    pub fn key_input_channel() -> std::sync::mpsc::Receiver<super::Key> {
        use std::{convert::TryInto, io};
        use termion::input::TermRead;

        let (key_send, key_receive) = std::sync::mpsc::sync_channel(0);
        std::thread::spawn(move || -> Result<(), io::Error> {
            for key in io::stdin().keys() {
                let key: Result<super::Key, _> = match continue_on_interrupt(key) {
                    Action::Continue => continue,
                    Action::Result(res) => res?.try_into(),
                };
                if let Ok(key) = key {
                    if key_send.send(key).is_err() {
                        break;
                    }
                }
            }
            Ok(())
        });
        key_receive
    }
}

#[cfg(any(feature = "termion", feature = "crossterm"))]
pub use _impl::*;
