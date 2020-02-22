use crate::{
    tree::{Key, Progress, ProgressState, ProgressStep, Value},
    tui::utils::{
        block_width, draw_text_nowrap, draw_text_nowrap_fn, rect, sanitize_offset,
        GraphemeCountWriter,
    },
};
use humantime::format_duration;
use std::{
    fmt,
    time::{Duration, SystemTime},
};
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};
use tui_react::fill_background;

const MIN_TREE_WIDTH: u16 = 20;

pub fn pane(entries: &[(Key, Value)], mut bound: Rect, offset: &mut u16, buf: &mut Buffer) {
    *offset = sanitize_offset(*offset, entries.len(), bound.height);
    let needs_overflow_line =
        if entries.len() > bound.height as usize || (*offset).min(entries.len() as u16) > 0 {
            bound.height = bound.height.saturating_sub(1);
            true
        } else {
            false
        };
    *offset = sanitize_offset(*offset, entries.len(), bound.height);

    if !entries.is_empty() {
        let column_width = bound.width / 2;
        let max_tree_draw_width = if column_width >= MIN_TREE_WIDTH {
            let prefix_area = Rect {
                width: column_width,
                ..bound
            };
            draw_tree(entries, buf, prefix_area, *offset)
        } else {
            0
        };

        {
            let max_tree_draw_width = max_tree_draw_width;
            let progress_area = rect::offset_x(bound, max_tree_draw_width);
            draw_progress(
                entries,
                buf,
                progress_area,
                if max_tree_draw_width == 0 {
                    false
                } else {
                    true
                },
                *offset,
            );
        }

        if needs_overflow_line {
            let overflow_rect = Rect {
                y: bound.height + 1,
                height: 1,
                ..bound
            };
            draw_overflow(
                entries,
                buf,
                overflow_rect,
                max_tree_draw_width,
                bound.height,
                *offset,
            );
        }
    }
}

pub fn headline(
    entries: &[(Key, Value)],
    duration_per_frame: Duration,
    buf: &mut Buffer,
    bound: Rect,
) {
    let (num_running_tasks, num_blocked_tasks, num_groups) = entries.iter().fold(
        (0, 0, 0),
        |(mut running, mut blocked, mut groups), (_key, Value { progress, .. })| {
            match progress.map(|p| p.state) {
                Some(ProgressState::Running) => running += 1,
                Some(ProgressState::Blocked(_)) => blocked += 1,
                None => groups += 1,
            }
            (running, blocked, groups)
        },
    );
    let text = format!(
        "{} {:3} running + {:3} blocked + {:3} groups = {} ",
        if duration_per_frame > Duration::from_secs(1) {
            format!(
                " Every {}s → {}",
                duration_per_frame.as_secs(),
                String::from_utf8_lossy(
                    &humantime::format_rfc3339_seconds(SystemTime::now())
                        .to_string()
                        .as_bytes()[.."2020-02-13T00:51:45".len()]
                )
            )
        } else {
            "".into()
        },
        num_running_tasks,
        num_blocked_tasks,
        num_groups,
        entries.len()
    );
    draw_text_nowrap(
        rect::snap_to_right(bound, block_width(&text) + 1),
        buf,
        text,
        None,
    );
}

struct ProgressFormat<'a>(&'a Option<Progress>, u16);

impl<'a> fmt::Display for ProgressFormat<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(p) => {
                match p.done_at {
                    Some(done_at) => write!(f, "{} / {}", p.step, done_at),
                    None => write!(f, "{}", p.step),
                }?;
                if let Some(unit) = p.unit {
                    write!(f, " {}", unit)?;
                }
                Ok(())
            }
            None => write!(f, "{:─<width$}", '─', width = self.1 as usize),
        }
    }
}

pub fn draw_progress(
    entries: &[(Key, Value)],
    buf: &mut Buffer,
    bound: Rect,
    draw_column_line: bool,
    offset: u16,
) {
    let title_spacing = 2u16 + 1; // 2 on the left, 1 on the right
    let column_line_width = if draw_column_line { 1 } else { 0 };
    let max_progress_label_width = entries
        .iter()
        .skip(offset as usize)
        .take(bound.height as usize)
        .map(|(_, Value { progress, .. })| progress)
        .fold(0, |state, progress| match progress {
            progress @ Some(_) => {
                use std::io::Write;
                let mut w = GraphemeCountWriter::default();
                write!(w, "{}", ProgressFormat(progress, 0)).expect("never fails");
                state.max(w.0)
            }
            None => state,
        });
    let max_title_width = entries
        .iter()
        .skip(offset as usize)
        .take(bound.height as usize)
        .fold(
            0,
            |state,
             (
                key,
                Value {
                    progress,
                    name: title,
                },
            )| match progress {
                None => state.max(block_width(title) + key.level() as u16 + title_spacing),
                Some(_) => state,
            },
        );

    for (
        line,
        (
            key,
            Value {
                progress,
                name: title,
            },
        ),
    ) in entries
        .iter()
        .skip(offset as usize)
        .take(bound.height as usize)
        .enumerate()
    {
        let line_bound = rect::line_bound(bound, line);
        let progress_text = format!(
            " {progress}",
            progress = ProgressFormat(progress, bound.width.saturating_sub(title_spacing))
        );

        draw_text_nowrap(line_bound, buf, rect::VERTICAL_LINE, None);

        let progress_rect = rect::offset_x(line_bound, column_line_width);
        match progress.map(|p| (p.fraction(), p.state, p.step)) {
            Some((Some(fraction), state, _step)) => {
                let mut progress_text = progress_text;
                if let ProgressState::Blocked(Some(eta)) = state {
                    let now = SystemTime::now();
                    if eta > now {
                        progress_text.push_str(&format!(
                            " → {} to unblock",
                            format_duration(eta.duration_since(now).expect("computation to work"))
                        ))
                    }
                }
                let (bound, style) =
                    draw_progress_bar_fn(buf, progress_rect, fraction, |fraction| {
                        if let ProgressState::Blocked(_) = state {
                            return Color::Red;
                        }
                        if fraction >= 0.8 {
                            Color::Green
                        } else {
                            Color::Yellow
                        }
                    });
                let style_fn = move |_t: &str, x: u16, _y: u16| {
                    if x < bound.right() {
                        style
                    } else {
                        Style::default()
                    }
                };
                draw_text_nowrap_fn(progress_rect, buf, progress_text, style_fn);
            }
            Some((None, state, step)) => {
                draw_text_nowrap(progress_rect, buf, progress_text, None);
                let bar_rect = rect::offset_x(line_bound, max_progress_label_width as u16);
                draw_spinner(
                    buf,
                    bar_rect,
                    step,
                    line,
                    if let ProgressState::Blocked(_) = state {
                        Color::Red
                    } else {
                        Color::White
                    },
                );
            }
            None => {
                let center_rect = rect::intersect(
                    Rect {
                        x: line_bound.x
                            + column_line_width
                            + (line_bound.width.saturating_sub(max_title_width as u16)) / 2,
                        width: max_title_width as u16,
                        ..line_bound
                    },
                    line_bound,
                );
                let title_text = format!(
                    " {:‧<prefix_count$} {} ",
                    "",
                    title,
                    prefix_count = key.level() as usize
                );
                draw_text_nowrap(progress_rect, buf, progress_text, None);
                draw_text_nowrap(center_rect, buf, title_text, None);
            }
        }
    }
}

fn draw_spinner(buf: &mut Buffer, bound: Rect, step: ProgressStep, seed: usize, color: Color) {
    if bound.width == 0 {
        return;
    }
    let step = step as usize;
    let x = bound.x + ((step + seed) % bound.width as usize) as u16;
    let width = 5;
    let bound = rect::intersect(Rect { x, width, ..bound }, bound);
    tui_react::fill_background(bound, buf, color);
}

fn draw_progress_bar_fn(
    buf: &mut Buffer,
    bound: Rect,
    fraction: f32,
    style: impl FnOnce(f32) -> Color,
) -> (Rect, Style) {
    if bound.width == 0 {
        return (Rect::default(), Style::default());
    }
    let fractional_progress_rect = Rect {
        width: ((bound.width as f32 * fraction).ceil() as u16).min(bound.width),
        ..bound
    };
    let color = style(fraction);
    tui_react::fill_background(fractional_progress_rect, buf, color);
    (
        fractional_progress_rect,
        Style::default().bg(color).fg(Color::Black),
    )
}

pub fn draw_tree(entries: &[(Key, Value)], buf: &mut Buffer, bound: Rect, offset: u16) -> u16 {
    let mut max_prefix_len = 0;
    for (
        line,
        (
            key,
            Value {
                progress,
                name: title,
            },
        ),
    ) in entries
        .iter()
        .skip(offset as usize)
        .take(bound.height as usize)
        .enumerate()
    {
        let line_bound = rect::line_bound(bound, line);
        let tree_prefix = format!(
            "{:>width$} {} ",
            if key.level() == 1 {
                "‧"
            } else {
                if progress.is_none() {
                    "…"
                } else {
                    "└"
                }
            },
            if progress.is_none() { "" } else { &title },
            width = key.level() as usize
        );
        max_prefix_len = max_prefix_len.max(draw_text_nowrap(line_bound, buf, tree_prefix, None));
    }
    max_prefix_len
}

pub fn draw_overflow<'a>(
    entries: &[(Key, Value)],
    buf: &mut Buffer,
    bound: Rect,
    label_offset: u16,
    num_entries_on_display: u16,
    offset: u16,
) {
    let (count, mut progress_fraction) = entries
        .iter()
        .take(offset as usize)
        .chain(
            entries
                .iter()
                .skip((offset + num_entries_on_display) as usize),
        )
        .fold(
            (0usize, 0f32),
            |(count, progress_fraction), (_key, value)| {
                let progress = value
                    .progress
                    .and_then(|p| p.fraction())
                    .unwrap_or_default();
                (count + 1, progress_fraction + progress)
            },
        );
    progress_fraction /= count as f32;
    let label = format!(
        "{} …{} skipped and {} more",
        if label_offset == 0 {
            ""
        } else {
            rect::VERTICAL_LINE
        },
        offset,
        entries
            .len()
            .saturating_sub((offset + num_entries_on_display + 1) as usize)
    );
    let (progress_rect, style) =
        draw_progress_bar_fn(buf, bound, progress_fraction, |_| Color::Green);

    let bg_color = Color::Red;
    fill_background(
        rect::offset_x(bound, progress_rect.right() - 1),
        buf,
        bg_color,
    );
    let color_text_according_to_progress = move |_g: &str, x: u16, _y: u16| {
        if x < progress_rect.right() {
            style
        } else {
            style.bg(bg_color)
        }
    };
    draw_text_nowrap_fn(
        rect::offset_x(bound, label_offset),
        buf,
        label,
        color_text_according_to_progress,
    );
    let help_text = "⇊ = d|↓ = j|⇈ = u|↑ = k ";
    draw_text_nowrap_fn(
        rect::snap_to_right(bound, block_width(help_text)),
        buf,
        help_text,
        color_text_according_to_progress,
    );
}
