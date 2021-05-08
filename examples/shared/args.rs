use argh::FromArgs;

#[derive(FromArgs)]
/// Reach new heights.
pub struct Options {
    /// if set, the terminal window will be animated to assure resizing works as expected.
    #[argh(switch, short = 'a')]
    pub animate_terminal_size: bool,

    /// if set, names of tasks will change rapidly, causing the delay at which column sizes are recalculated to show
    #[argh(switch, short = 'c')]
    pub changing_names: bool,

    /// the amount of frames to show per second, can be below zero, e.g.
    /// 0.25 shows a frame every 4 seconds.
    #[argh(option, default = "10.0")]
    pub fps: f32,

    /// if set, recompute the column width of the task tree only every given frame. Otherwise the width will be recomputed every frame.
    ///
    /// Use this if there are many short-running tasks with varying names paired with high refresh rates of multiple frames per second to
    /// stabilize the appearance of the TUI.
    ///
    /// For example, setting the value to 40 will with a frame rate of 20 per second will recompute the column width to fit all task names
    /// every 2 seconds.
    #[argh(option, short = 'r')]
    pub recompute_column_width_every_nth_frame: Option<usize>,

    /// the amount of scrollback for task messages.
    #[argh(option, default = "80")]
    pub message_scrollback_buffer_size: usize,

    /// the amount of pooled work chunks that can be created at most
    #[argh(option, default = "16")]
    pub pooled_work_max: usize,

    /// the amount of pooled work chunks that should at least be created
    #[argh(option, default = "6")]
    pub pooled_work_min: usize,

    /// multiplies the speed at which tasks seem to be running. Driving this down makes the TUI easier on the eyes
    /// Defaults to 1.0. A valud of 0.5 halves the speed.
    #[argh(option, short = 's', default = "1.0")]
    pub speed_multitplier: f32,

    /// for 'line' renderer: Determines the amount of seconds that the progress has to last at least until we see the first progress.
    #[argh(option)]
    pub line_initial_delay: Option<f32>,

    /// for 'line' renderer: If true, timestamps will be displayed for each printed message.
    #[argh(switch)]
    pub line_timestamp: bool,

    /// for 'line' renderer: The first level to display, defaults to 0
    #[argh(option)]
    pub line_start: Option<prodash::progress::key::Level>,

    /// for 'line' renderer: Amount of columns we should draw into. If unset, the whole width of the terminal.
    #[argh(option)]
    pub line_column_count: Option<u16>,

    /// for 'line' renderer: The first level to display, defaults to 1
    #[argh(option)]
    pub line_end: Option<prodash::progress::key::Level>,

    /// if set (default: false), we will stop running the TUI once there the list of drawable progress items is empty.
    #[argh(switch)]
    pub stop_if_empty_progress: bool,

    /// set the renderer to use, defaults to "tui", and furthermore allows "line" and "log".
    ///
    /// If set ot "log", there will only be logging. Set 'RUST_LOG=info' before running the program to see them.
    #[argh(option, short = 'R')]
    pub renderer: Option<String>,

    /// has not effect - use the NO_COLOR environment variable instead.
    #[argh(switch)]
    pub no_line_color: bool,
}
