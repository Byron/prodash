use crate::tree;
use ansi_term::Color;
use std::{io, ops::RangeInclusive};

#[derive(Default)]
pub struct State {
    tree: Vec<(tree::Key, tree::Value)>,
    messages: Vec<tree::Message>,
    from_copying: Option<tree::MessageCopyState>,
}

pub struct Options {
    pub level_filter: Option<RangeInclusive<tree::Level>>,
    pub keep_running_if_progress_is_empty: bool,
    pub output_is_terminal: bool,
    pub colored: bool,
}

mod utils {
    use ansi_term::{ANSIGenericString, Style};

    pub struct Brush {
        may_paint: bool,
        style: Option<Style>,
    }

    impl Brush {
        pub fn new(colored: bool) -> Self {
            Brush {
                may_paint: colored,
                style: None,
            }
        }

        pub fn style(&mut self, style: Style) -> &mut Self {
            self.style = Some(style);
            self
        }

        #[must_use]
        pub fn paint<'a, I, S: 'a + ToOwned + ?Sized>(&mut self, input: I) -> ANSIGenericString<'a, S>
        where
            I: Into<std::borrow::Cow<'a, S>>,
            <S as ToOwned>::Owned: std::fmt::Debug,
        {
            match (self.may_paint, self.style.take()) {
                (true, Some(style)) => style.paint(input),
                (_, Some(_)) | (_, None) => ANSIGenericString::from(input),
            }
        }
    }
}

fn messages(_out: &mut impl io::Write, messages: &[tree::Message], colored: bool) -> io::Result<()> {
    let mut brush = utils::Brush::new(colored);
    fn to_color(level: tree::MessageLevel) -> Color {
        use tree::MessageLevel::*;
        match level {
            Info => Color::White,
            Success => Color::Green,
            Failure => Color::Red,
        }
    }
    for tree::Message {
        time: _,
        level,
        origin,
        message,
    } in messages
    {
        writeln!(
            _out,
            "{}→{}",
            brush.style(Color::Yellow.dimmed()).paint(origin),
            brush.style(to_color(*level).bold()).paint(message)
        )?;
    }
    Ok(())
}

pub fn lines(out: &mut impl io::Write, progress: &tree::Root, state: &mut State, config: &Options) -> io::Result<()> {
    progress.sorted_snapshot(&mut state.tree);
    if !config.keep_running_if_progress_is_empty && state.tree.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "stop as progress is empty"));
    }
    state.from_copying = Some(progress.copy_new_messages(&mut state.messages, state.from_copying.take()));
    messages(out, &state.messages, config.colored)?;
    if config.output_is_terminal {
        let level_range = config
            .level_filter
            .clone()
            .unwrap_or(RangeInclusive::new(0, tree::Level::max_value()));
        for (_key, _progress) in state.tree.iter().filter(|(k, _)| level_range.contains(&k.level())) {
            unimplemented!("drawing to be done")
        }
    }
    Ok(())
}
