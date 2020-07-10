use crate::tree;
use crosstermion::ansi_term::{ANSIString, ANSIStrings, Color, Style};
use std::{io, ops::RangeInclusive};
use unicode_width::UnicodeWidthStr;

#[derive(Default)]
pub struct State {
    tree: Vec<(tree::Key, tree::Value)>,
    messages: Vec<tree::Message>,
    for_next_copy: Option<tree::MessageCopyState>,
    max_message_origin_size: usize,
    /// The amount of blocks per line we have written last time.
    blocks_per_line: std::collections::VecDeque<u16>,
    /// Amount of times we drew so far
    ticks: usize,
}

pub struct Options {
    pub level_filter: Option<RangeInclusive<tree::Level>>,
    pub keep_running_if_progress_is_empty: bool,
    pub output_is_terminal: bool,
    pub colored: bool,
    pub timestamp: bool,
    pub hide_cursor: bool,
}

fn messages(out: &mut impl io::Write, state: &mut State, colored: bool, timestamp: bool) -> io::Result<()> {
    let mut brush = crosstermion::color::Brush::new(colored);
    fn to_color(level: tree::MessageLevel) -> Color {
        use tree::MessageLevel::*;
        match level {
            Info => Color::White,
            Success => Color::Green,
            Failure => Color::Red,
        }
    }
    let mut tokens: Vec<ANSIString<'_>> = Vec::with_capacity(6);
    for tree::Message {
        time,
        level,
        origin,
        message,
    } in &state.messages
    {
        tokens.clear();
        let blocks_drawn_during_previous_tick = state.blocks_per_line.pop_front().unwrap_or(0);
        let message_block_len = origin.width();
        state.max_message_origin_size = state.max_message_origin_size.max(message_block_len);

        let color = to_color(*level);
        tokens.push(" ".into());
        if timestamp {
            tokens.push(
                brush
                    .style(color.dimmed().on(Color::Yellow))
                    .paint(crate::time::format_time_for_messages(*time)),
            );
            tokens.push(Style::new().paint(" "));
        } else {
            tokens.push("".into());
        };
        tokens.push(brush.style(Style::default().dimmed()).paint(format!(
            "{:>fill_size$}{}",
            "",
            origin,
            fill_size = state.max_message_origin_size - message_block_len,
        )));
        tokens.push(" ".into());
        tokens.push(brush.style(color.bold()).paint(message));
        let message_block_count = block_count_sans_ansi_codes(&tokens);
        write!(out, "{}", ANSIStrings(tokens.as_slice()))?;

        if blocks_drawn_during_previous_tick > message_block_count {
            newline_with_overdraw(out, &tokens, blocks_drawn_during_previous_tick)?;
        } else {
            writeln!(out)?;
        }
    }
    Ok(())
}

pub fn all(out: &mut impl io::Write, progress: &tree::Root, state: &mut State, config: &Options) -> io::Result<()> {
    progress.sorted_snapshot(&mut state.tree);
    if !config.keep_running_if_progress_is_empty && state.tree.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "stop as progress is empty"));
    }
    state.for_next_copy = Some(progress.copy_new_messages(&mut state.messages, state.for_next_copy.take()));
    messages(out, state, config.colored, config.timestamp)?;

    if config.output_is_terminal {
        let level_range = config
            .level_filter
            .clone()
            .unwrap_or(RangeInclusive::new(0, tree::Level::max_value()));
        let lines_to_be_drawn = state
            .tree
            .iter()
            .filter(|(k, _)| level_range.contains(&k.level()))
            .count();
        if state.blocks_per_line.len() < lines_to_be_drawn {
            state.blocks_per_line.resize(lines_to_be_drawn, 0);
        }
        let mut tokens: Vec<ANSIString<'_>> = Vec::with_capacity(4);
        for ((key, progress), ref mut blocks_in_last_iteration) in state
            .tree
            .iter()
            .filter(|(k, _)| level_range.contains(&k.level()))
            .zip(state.blocks_per_line.iter_mut())
        {
            format_progress(key, progress, state.ticks, &mut tokens);
            write!(out, "{}", ANSIStrings(tokens.as_slice()))?;

            **blocks_in_last_iteration = newline_with_overdraw(out, &tokens, **blocks_in_last_iteration)?;
        }
        // overwrite remaining lines that we didn't touch naturally
        let lines_drawn = lines_to_be_drawn;
        if state.blocks_per_line.len() > lines_drawn {
            for blocks_in_last_iteration in state.blocks_per_line.iter().skip(lines_drawn) {
                writeln!(out, "{:>width$}", "", width = *blocks_in_last_iteration as usize)?;
            }
            // Move cursor back to end of the portion we have actually drawn
            crosstermion::execute!(out, crosstermion::cursor::MoveUp(state.blocks_per_line.len() as u16))?;
            state.blocks_per_line.resize(lines_drawn, 0);
        } else {
            crosstermion::execute!(out, crosstermion::cursor::MoveUp(lines_drawn as u16))?;
        }
    }
    state.ticks += 1;
    Ok(())
}

/// Must be called directly after `tokens` were drawn, without newline. Takes care of adding the newline.
fn newline_with_overdraw(
    out: &mut impl io::Write,
    tokens: &[ANSIString<'_>],
    blocks_in_last_iteration: u16,
) -> io::Result<u16> {
    let current_block_count = block_count_sans_ansi_codes(&tokens);
    if blocks_in_last_iteration > current_block_count {
        // fill to the end of line to overwrite what was previously there
        writeln!(
            out,
            "{:>width$}",
            "",
            width = (blocks_in_last_iteration - current_block_count) as usize
        )?;
    } else {
        writeln!(out)?;
    };
    Ok(current_block_count)
}

fn block_count_sans_ansi_codes(strings: &[ANSIString<'_>]) -> u16 {
    strings.iter().map(|s| s.width() as u16).sum()
}

fn draw_progress_bar<'a>(_p: &tree::Progress, _style: Style, buf: &mut Vec<ANSIString<'a>>) {
    // [=====================================================> ]
    buf.push("[".into());
    buf.push("]".into());
}

fn progress_style(p: &tree::Progress) -> Style {
    use tree::ProgressState::*;
    match p.state {
        Running => if let Some(fraction) = p.fraction() {
            if fraction > 0.8 {
                Color::Green
            } else {
                Color::Yellow
            }
        } else {
            Color::White
        }
        .normal(),
        Halted(_, _) => Color::Red.dimmed(),
        Blocked(_, _) => Color::Red.normal(),
    }
}

fn format_progress<'a>(key: &tree::Key, value: &'a tree::Value, _ticks: usize, buf: &mut Vec<ANSIString<'a>>) {
    const LINE_LENGTH: u16 = 80;
    buf.clear();

    buf.push(Style::new().paint(format!("{:>level$}", "", level = key.level() as usize)));
    match value.progress {
        Some(progress) => {
            let style = progress_style(&progress);
            buf.push(Color::Cyan.bold().paint(&value.name));
            buf.push(" ".into());
            let _blocks_left = LINE_LENGTH.saturating_sub(block_count_sans_ansi_codes(buf.as_slice()));
            draw_progress_bar(&progress, style, buf);
        }
        None => {
            // headline only
            buf.push(Color::White.bold().paint(&value.name));
        }
    }
}
