use futures::{task::Poll, FutureExt};
use futures_timer::Delay;
use std::{io::Error, time::Duration};
use tui::{buffer::Buffer, layout::Rect, style::Style};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Returns a stream of 'ticks', each being duration `dur` apart.
///
/// Can be useful to provide the TUI with additional events in regular intervals,
/// when using the [`tui::render_with_input(…events)`](./fn.render_with_input.html) function.
pub fn ticker(dur: Duration) -> impl futures::Stream<Item = ()> {
    let mut delay = Delay::new(dur);
    futures::stream::poll_fn(move |ctx| {
        let res = delay.poll_unpin(ctx);
        match res {
            Poll::Pending => Poll::Pending,
            Poll::Ready(_) => {
                delay.reset(dur);
                Poll::Ready(Some(()))
            }
        }
    })
}

pub fn sanitize_offset(offset: u16, num_items: usize, num_displayable_lines: u16) -> u16 {
    offset.min((num_items.saturating_sub(num_displayable_lines as usize)) as u16)
}

#[derive(Default)]
pub struct GraphemeCountWriter(pub usize);

impl std::io::Write for GraphemeCountWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.0 += String::from_utf8_lossy(buf).graphemes(true).count();
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

// TODO: put this in tui-react
pub fn draw_text_nowrap<'a>(
    bound: Rect,
    buf: &mut Buffer,
    t: impl AsRef<str>,
    s: impl Into<Option<Style>>,
) -> u16 {
    let s = s.into();
    let t = t.as_ref();
    let mut graphemes = t.graphemes(true);
    let mut total_width = 0;
    {
        let mut ellipsis_candidate_x = None;
        let mut x_offset = 0;
        for (g, mut x) in graphemes.by_ref().zip(bound.left()..bound.right()) {
            let width = g.width();
            total_width += width;

            x += x_offset;
            let cell = buf.get_mut(x, bound.y);
            if x + 1 == bound.right() {
                ellipsis_candidate_x = Some(x);
            }
            cell.symbol = g.into();
            if let Some(s) = s {
                cell.style = s;
            }

            x_offset += (width - 1) as u16;
            if x + x_offset >= bound.right() {
                break;
            }
            let x = x as usize;
            for x in x + 1..x + width {
                let i = buf.index_of(x as u16, bound.y);
                buf.content[i].reset();
            }
        }
        if let (Some(_), Some(x)) = (graphemes.next(), ellipsis_candidate_x) {
            buf.get_mut(x, bound.y).symbol = "…".into();
        }
    }
    total_width as u16
}

// TODO: put this in tui-react
pub fn draw_text_nowrap_fn(
    bound: Rect,
    buf: &mut Buffer,
    t: impl AsRef<str>,
    mut s: impl FnMut(&str, u16, u16) -> Style,
) {
    if bound.width == 0 {
        return;
    }
    for (g, x) in t.as_ref().graphemes(true).zip(bound.left()..bound.right()) {
        let cell = buf.get_mut(x, bound.y);
        cell.symbol = g.into();
        cell.style = s(&cell.symbol, x, bound.y);
    }
}

pub fn block_width(s: &str) -> u16 {
    s.graphemes(true).map(|g| g.width()).sum::<usize>() as u16
}

pub mod rect {
    use tui::layout::Rect;

    pub const VERTICAL_LINE: &str = "│";

    /// A safe version of Rect::intersection that doesn't suffer from underflows
    pub fn intersect(lhs: Rect, rhs: Rect) -> Rect {
        let x1 = lhs.x.max(rhs.x);
        let y1 = lhs.y.max(rhs.y);
        let x2 = lhs.right().min(rhs.right());
        let y2 = lhs.bottom().min(rhs.bottom());
        Rect {
            x: x1,
            y: y1,
            width: x2.saturating_sub(x1),
            height: y2.saturating_sub(y1),
        }
    }

    pub fn offset_x(r: Rect, offset: u16) -> Rect {
        Rect {
            x: r.x + offset,
            width: r.width.saturating_sub(offset),
            ..r
        }
    }

    pub fn snap_to_right(bound: Rect, new_width: u16) -> Rect {
        offset_x(bound, bound.width.saturating_sub(new_width))
    }

    pub fn line_bound(bound: Rect, line: usize) -> Rect {
        Rect {
            y: bound.y + line as u16,
            height: 1,
            ..bound
        }
    }
}
