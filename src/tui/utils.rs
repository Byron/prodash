use futures::{task::Poll, FutureExt};
use futures_timer::Delay;
use std::time::Duration;

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

pub const VERTICAL_LINE: &str = "│";

pub use tui_react::util::*;
pub use tui_react::{draw_text_nowrap_fn, draw_text_with_ellipsis_nowrap};
