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

#[cfg(feature = "crossterm")]
mod _impl {
    use super::Key;

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

    /// Return a receiver of user input events to avoid blocking the main thread.
    ///
    /// Requires feature `input-thread`
    #[cfg(feature = "input-thread")]
    pub fn key_input_channel() -> crossbeam_channel::Receiver<Key> {
        use std::convert::TryInto;

        let (key_send, key_receive) = crossbeam_channel::bounded::<Key>(0);
        std::thread::spawn(move || -> Result<(), std::io::Error> {
            loop {
                let event = crossterm::event::read().map_err(crate::crossterm::into_io_error)?;
                match event {
                    crossterm::event::Event::Key(key) => {
                        let key: Result<Key, _> = key.try_into();
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
    #[cfg(feature = "input-async")]
    pub fn key_input_stream() -> impl futures_util::stream::Stream<Item = Key> {
        use futures_util::StreamExt;
        use std::convert::TryFrom;
        crossterm::event::EventStream::new()
            .filter_map(|r| futures_util::future::ready(r.ok()))
            .filter_map(|e| {
                futures_util::future::ready(match e {
                    crossterm::event::Event::Key(key) => Key::try_from(key).ok(),
                    _ => None,
                })
            })
    }
}

#[cfg(all(feature = "termion", not(feature = "crossterm")))]
mod _impl {
    use super::Key;

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

    /// Return a stream of user input events.
    ///
    /// Requires feature `futures-channel`
    #[cfg(feature = "input-async")]
    pub fn key_input_stream() -> impl futures_util::stream::Stream<Item = Key> {
        use futures_util::SinkExt;
        use std::{convert::TryInto, io};
        use termion::input::TermRead;

        let (mut key_send, key_receive) = futures_channel::mpsc::channel::<Key>(1);
        // This brings blocking key-handling into the async world
        std::thread::spawn(move || -> Result<(), io::Error> {
            for key in io::stdin().keys() {
                let key: Result<Key, _> = key?.try_into();
                if let Ok(key) = key {
                    if futures_executor::block_on(key_send.send(key)).is_err() {
                        break;
                    }
                }
            }
            Ok(())
        });
        key_receive
    }

    #[cfg(feature = "input-thread")]
    pub fn key_input_channel() -> crossbeam_channel::Receiver<Key> {
        use std::{convert::TryInto, io};
        use termion::input::TermRead;

        let (key_send, key_receive) = crossbeam_channel::bounded::<Key>(1);
        std::thread::spawn(move || -> Result<(), io::Error> {
            for key in io::stdin().keys() {
                let key: Result<Key, _> = key?.try_into();
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
