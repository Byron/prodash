pub mod input {
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
        use crate::input::Key;

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
    }

    #[cfg(all(feature = "termion", not(feature = "crossterm")))]
    pub mod _impl {
        use crate::tui::input::Key;
        use std::convert::TryInto;
        use termion::event::Key;

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
}
