use crate::tree;
use std::{io, ops::RangeInclusive, time::Duration};

#[cfg(all(
    feature = "line-renderer",
    not(any(feature = "line-renderer-crossterm", feature = "line-renderer-termion"))
))]
compile_error!("Please choose either one of these features: 'line-renderer-crossterm' or 'line-renderer-termion'");

#[derive(Clone)]
pub struct Options {
    /// If set, specify all levels that should be shown. Otherwise all available levels are shown.
    ///
    /// This is useful to filter out high-noise lower level progress items in the tree.
    pub level_filter: Option<RangeInclusive<tree::Level>>,

    /// If set, progress will only actually be shown after the given duration.
    ///
    /// This option can be useful to not enforce progress for short actions, causing it to flicker.
    /// Please note that this won't affect display of messages, which are simply logged.
    pub initial_delay: Option<Duration>,

    /// The amount of frames to draw per second. If below 1.0, it determines the amount of seconds between the frame.
    ///
    /// *e.g.* 1.0/4.0 is one frame every 4 seconds.
    pub frames_per_second: f32,

    /// If true (default: false), we will keep waiting for progress even after we encountered an empty list of drawable progress items.
    ///
    /// Please note that you should add at least one item to the `prodash::Tree` before launching the application or else
    /// risk a race causing nothing to be rendered at all.
    pub keep_running_if_progress_is_empty: bool,
}

pub struct JoinHandle {
    inner: Option<std::thread::JoinHandle<io::Result<()>>>,
    connect: Option<flume::Sender<Event>>,
}

impl Drop for JoinHandle {
    fn drop(&mut self) {
        if let Some(chan) = self.connect.take() {
            chan.send(Event::Quit).ok();
        }
        self.inner.take().and_then(|h| h.join().ok());
    }
}

enum Event {
    Tick,
    Quit,
}

const FPS_NEEDED_TO_SHUTDOWN_FAST_ENOUGH: f32 = 4.0;

#[derive(Default)]
struct State {
    tree: Vec<(tree::Key, tree::Value)>,
}

fn draw(
    out: &mut impl io::Write,
    progress: &tree::Root,
    state: &mut State,
    config: &Options,
) -> io::Result<()> {
    progress.sorted_snapshot(&mut state.tree);
    if !config.keep_running_if_progress_is_empty && state.tree.is_empty() {
        return Ok(());
    }
    unimplemented!("drawing to be done")
}

pub fn render(
    mut out: impl io::Write + Send + 'static,
    progress: tree::Root,
    config: Options,
) -> JoinHandle {
    let (quit_send, quit_recv) = flume::bounded::<Event>(0);
    let join = std::thread::spawn(move || {
        let mut state = State::default();
        if config.frames_per_second >= FPS_NEEDED_TO_SHUTDOWN_FAST_ENOUGH {
            loop {
                if let Err(flume::TryRecvError::Disconnected) = quit_recv.try_recv() {
                    break;
                }
                draw(&mut out, &progress, &mut state, &config)?;
                std::thread::sleep(Duration::from_secs_f32(1.0 / config.frames_per_second));
            }
        } else {
            let (tick_send, tick_recv) = flume::bounded::<Event>(0);
            let secs = 1.0 / config.frames_per_second;
            std::thread::spawn(move || loop {
                if tick_send.send(Event::Tick).is_err() {
                    break;
                }
                std::thread::sleep(Duration::from_secs_f32(secs));
            });

            let mut selector = flume::Selector::new()
                .recv(&quit_recv, |_res| Event::Quit)
                .recv(&tick_recv, |_res| Event::Tick);
            while let Event::Tick = selector.wait() {
                draw(&mut out, &progress, &mut state, &config)?;
            }
        }
        Ok(())
    });

    JoinHandle {
        inner: Some(join),
        connect: Some(quit_send),
    }
}
