use crate::tree;
use crosstermion::ansi_term::Color;
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
    pub timestamp: bool,
}

fn messages(out: &mut impl io::Write, messages: &[tree::Message], colored: bool, timestamp: bool) -> io::Result<()> {
    let mut brush = crosstermion::color::Brush::new(colored);
    fn to_color(level: tree::MessageLevel) -> Color {
        use tree::MessageLevel::*;
        match level {
            Info => Color::White,
            Success => Color::Green,
            Failure => Color::Red,
        }
    }
    for tree::Message {
        time,
        level,
        origin,
        message,
    } in messages
    {
        let color = to_color(*level);
        writeln!(
            out,
            " {}{} {}",
            if timestamp {
                format!(
                    "{} ",
                    brush
                        .style(color.dimmed().on(Color::Yellow))
                        .paint(crate::time::format_time_for_messages(*time))
                )
            } else {
                "".into()
            },
            brush
                .style(crosstermion::ansi_term::Style::default().dimmed())
                .paint(origin),
            brush.style(color.bold()).paint(message)
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
    messages(out, &state.messages, config.colored, config.timestamp)?;
    if config.output_is_terminal {
        let level_range = config
            .level_filter
            .clone()
            .unwrap_or(RangeInclusive::new(0, tree::Level::max_value()));
        for (_key, _progress) in state.tree.iter().filter(|(k, _)| level_range.contains(&k.level())) {
            // unimplemented!("drawing to be done")
        }
    }
    Ok(())
}
