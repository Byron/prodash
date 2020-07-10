use crate::{line::draw, tree};
use std::{
    io,
    ops::RangeInclusive,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

#[derive(Clone)]
pub struct Options {
    /// If true, _(default true)_, we assume the output stream belongs to a terminal.
    ///
    /// If false, we won't print any live progress, only log messages.
    pub output_is_terminal: bool,

    /// If true, _(default: true)_ we will display color. You should use `output_is_terminal && crosstermion::should_colorize()`
    ///
    /// Please note that you can enforce color even if the output stream is not connected to a terminal by setting
    /// this field to true.
    pub colored: bool,

    /// If true, _(default: false)_, a timestamp will be shown before each message.
    pub timestamp: bool,

    /// The amount of columns and rows to use for drawing. Defaults to (80, 20).
    pub terminal_dimension: (u16, u16),

    /// If true, _(default: false)_, the cursor will be hidden for a more visually appealing display.
    ///
    /// Please note that you must make sure the line renderer is properly shut down to restore the previous cursor
    /// settings. See the `ctrlc` documentation in the README for more information.
    pub hide_cursor: bool,

    /// If set, specify all levels that should be shown. Otherwise all available levels are shown.
    ///
    /// This is useful to filter out high-noise lower level progress items in the tree.
    pub level_filter: Option<RangeInclusive<tree::Level>>,

    /// If set, progress will only actually be shown after the given duration. Log messages will always be shown without delay.
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

impl Default for Options {
    fn default() -> Self {
        Options {
            output_is_terminal: true,
            colored: true,
            timestamp: false,
            terminal_dimension: (80, 20),
            hide_cursor: false,
            level_filter: None,
            initial_delay: None,
            frames_per_second: FPS_NEEDED_TO_SHUTDOWN_FAST_ENOUGH,
            keep_running_if_progress_is_empty: false,
        }
    }
}

/// A handle to the render thread, which when dropped will instruct it to stop showing progress.
pub struct JoinHandle {
    inner: Option<std::thread::JoinHandle<io::Result<()>>>,
    connection: flume::Sender<Event>,
    // If we disconnect before sending a Quit event, the selector continuously informs about the 'Disconnect' state
    disconnected: bool,
}

impl JoinHandle {
    /// `detach()` and `forget()` to remove any effects associated with this handle.
    pub fn detach(mut self) {
        self.disconnect();
        self.forget();
    }
    /// Remove the handles capability to instruct the render thread to stop.
    pub fn disconnect(&mut self) {
        self.disconnected = true;
    }
    /// Remove the handles capability to `join()` by forgetting the threads handle
    pub fn forget(&mut self) {
        self.inner.take();
    }
    /// Wait for the thread to shutdown naturally, for example because there is no more progress to display
    pub fn wait(mut self) {
        self.inner.take().and_then(|h| h.join().ok());
    }
    /// Send the signal to shutdown and wait for the thread to be shutdown.
    pub fn shutdown_and_wait(mut self) {
        if !self.disconnected {
            self.connection.send(Event::Quit).ok();
        }
        self.inner.take().and_then(|h| h.join().ok());
    }
}

impl Drop for JoinHandle {
    fn drop(&mut self) {
        if !self.disconnected {
            self.connection.send(Event::Quit).ok();
        }
    }
}

#[derive(Debug)]
enum Event {
    Tick,
    Quit,
}

const FPS_NEEDED_TO_SHUTDOWN_FAST_ENOUGH: f32 = 6.0;

pub fn render(mut out: impl io::Write + Send + 'static, progress: tree::Root, config: Options) -> JoinHandle {
    let Options {
        output_is_terminal,
        colored,
        timestamp,
        level_filter,
        terminal_dimension: column_count,
        initial_delay,
        frames_per_second,
        keep_running_if_progress_is_empty,
        hide_cursor,
    } = config;
    let config = draw::Options {
        output_is_terminal,
        terminal_dimensions: column_count,
        colored,
        timestamp,
        keep_running_if_progress_is_empty,
        level_filter,
        hide_cursor,
    };

    let (quit_send, quit_recv) = flume::unbounded::<Event>();
    let show_cursor = possibly_hide_cursor(&mut out, quit_send.clone(), hide_cursor);
    static SHOW_PROGRESS: AtomicBool = AtomicBool::new(false);

    let handle = std::thread::spawn(move || {
        {
            let initial_delay = initial_delay.unwrap_or_else(Duration::default);
            SHOW_PROGRESS.store(initial_delay == Duration::default(), Ordering::Relaxed);
            if !SHOW_PROGRESS.load(Ordering::Relaxed) {
                std::thread::spawn(move || {
                    std::thread::sleep(initial_delay);
                    SHOW_PROGRESS.store(true, Ordering::Relaxed);
                });
            }
        }

        let mut state = draw::State::default();
        if frames_per_second >= FPS_NEEDED_TO_SHUTDOWN_FAST_ENOUGH {
            loop {
                if let Ok(Event::Quit) = quit_recv.try_recv() {
                    break;
                }
                draw::all(
                    &mut out,
                    &progress,
                    SHOW_PROGRESS.load(Ordering::Relaxed),
                    &mut state,
                    &config,
                )?;
                std::thread::sleep(Duration::from_secs_f32(1.0 / frames_per_second));
            }
        } else {
            let (tick_send, tick_recv) = flume::unbounded::<Event>();
            let secs = 1.0 / frames_per_second;
            let _ticker = std::thread::spawn(move || loop {
                if tick_send.send(Event::Tick).is_err() {
                    break;
                }
                std::thread::sleep(Duration::from_secs_f32(secs));
            });

            let mut selector = flume::Selector::new()
                .recv(&quit_recv, |res| {
                    if let Ok(Event::Quit) = res {
                        Event::Quit
                    } else {
                        Event::Tick
                    }
                })
                .recv(&tick_recv, |_res| Event::Tick);
            while let Event::Tick = selector.wait() {
                draw::all(
                    &mut out,
                    &progress,
                    SHOW_PROGRESS.load(Ordering::Relaxed),
                    &mut state,
                    &config,
                )?;
            }
        }

        if show_cursor {
            crosstermion::execute!(out, crosstermion::cursor::Show).ok();
        }
        Ok(())
    });

    JoinHandle {
        inner: Some(handle),
        connection: quit_send,
        disconnected: false,
    }
}

// Not all configurations actually need it to be mut, but those with the 'ctrlc' feature do
#[allow(unused_mut)]
fn possibly_hide_cursor(out: &mut impl io::Write, quit_send: flume::Sender<Event>, mut hide_cursor: bool) -> bool {
    #[cfg(not(feature = "ctrlc"))]
    drop(quit_send);

    #[cfg(feature = "ctrlc")]
    if hide_cursor {
        hide_cursor = ctrlc::set_handler(move || drop(quit_send.send(Event::Quit).ok())).is_ok();
    }

    if hide_cursor {
        crosstermion::execute!(out, crosstermion::cursor::Hide).is_ok()
    } else {
        false
    }
}
