#![deny(unsafe_code)]

#[cfg(not(feature = "tui-renderer"))]
compile_error!(
    "The `tui-renderer` feature must be set, along with either `tui-renderer-crossterm` or `tui-renderer-termion`"
);
#[cfg(not(any(feature = "tui-renderer-crossterm", feature = "tui-renderer-termion")))]
compile_error!("Please set either the 'tui-renderer-crossterm' or 'tui-renderer-termion' feature whne using the 'tui-renderer'");

fn main() -> Result {
    env_logger::init();

    let args: arg::Options = argh::from_env();
    smol::run(work_forever(args))
}

async fn work_forever(mut args: arg::Options) -> Result {
    let progress = prodash::TreeOptions {
        message_buffer_capacity: args.message_scrollback_buffer_size,
        ..prodash::TreeOptions::default()
    }
    .create();
    // Now we should handle signals to be able to cleanup properly
    let speed = args.speed_multitplier;
    let changing_names = args.changing_names;

    let renderer = args.renderer.take().unwrap_or("tui".into());
    let mut gui_handle = if renderer == "log" {
        let never_ending = smol::Task::spawn(futures_util::future::pending::<()>());
        Some(never_ending.boxed())
    } else {
        Some(
            launch_ambient_gui(progress.clone(), &renderer, args)
                .unwrap()
                .boxed(),
        )
    };

    loop {
        let local_work = new_chunk_of_work(
            NestingLevel(thread_rng().gen_range(0, Key::max_level())),
            progress.clone(),
            speed,
            changing_names,
        )
        .boxed_local();
        let pooled_work = (0..thread_rng().gen_range(6, 16usize)).map(|_| {
            smol::Task::spawn(new_chunk_of_work(
                NestingLevel(thread_rng().gen_range(0, Key::max_level())),
                progress.clone(),
                speed,
                changing_names,
            ))
            .boxed_local()
        });

        match futures_util::future::select(
            join_all(std::iter::once(local_work).chain(pooled_work)),
            gui_handle.take().expect("gui handle"),
        )
        .await
        {
            Either::Left((_workblock_result, running_gui)) => {
                gui_handle = Some(running_gui);
                continue;
            }
            Either::Right(_gui_shutdown) => break,
        }
    }

    if let Some(gui) = gui_handle {
        // gui.cancel();
        gui.await;
    }
    Ok(())
}

fn launch_ambient_gui(
    progress: Tree,
    renderer: &str,
    args: arg::Options,
) -> std::result::Result<smol::Task<()>, std::io::Error> {
    let mut ticks: usize = 0;
    let mut interruptible = true;
    let render_fut = match renderer {
        "line" => unimplemented!("line"),
        "tui" => tui::render_with_input(
            std::io::stdout(),
            progress,
            tui::Options {
                title: TITLES.choose(&mut thread_rng()).copied().unwrap().into(),
                frames_per_second: args.fps,
                recompute_column_width_every_nth_frame: args.recompute_column_width_every_nth_frame,
                redraw_only_on_state_change: true,
                ..tui::Options::default()
            },
            futures_util::stream::select(
                window_resize_stream(args.animate_terminal_size),
                ticker(Duration::from_secs_f32((1.0 / args.fps).max(1.0))).map(move |_| {
                    ticks += 1;
                    if ticks % 2 == 0 {
                        let is_interruptible = interruptible;
                        interruptible = !interruptible;
                        return if is_interruptible {
                            Event::SetInterruptMode(Interrupt::Instantly)
                        } else {
                            Event::SetInterruptMode(Interrupt::Deferred)
                        };
                    }
                    if thread_rng().gen_bool(0.5) {
                        Event::SetTitle(TITLES.choose(&mut thread_rng()).unwrap().to_string())
                    } else {
                        Event::SetInformation(generate_statistics())
                    }
                }),
            ),
        )?,
        _ => panic!("Unknown renderer: '{}'", renderer),
    };
    let handle = smol::Task::spawn(render_fut.map(|_| ()));
    Ok(handle)
}

async fn work_item(mut progress: Item, speed: f32, changing_names: bool) {
    let max: u8 = thread_rng().gen_range(25, 125);
    progress.init(
        if max > WORK_STEPS_NEEDED_FOR_UNBOUNDED_TASK {
            None
        } else {
            Some(max.into())
        },
        if (max as usize % UNITS.len() + 1) == 0 {
            None
        } else {
            UNITS.choose(&mut thread_rng()).copied()
        },
    );

    for step in 0..max {
        progress.set(step as u32);
        let delay_ms = if thread_rng().gen_bool(CHANCE_TO_BLOCK_PER_STEP) {
            let eta = if thread_rng().gen_bool(CHANCE_TO_SHOW_ETA) {
                Some(SystemTime::now().add(Duration::from_millis(LONG_WORK_DELAY_MS)))
            } else {
                None
            };
            if thread_rng().gen_bool(0.5) {
                progress.halted(REASONS.choose(&mut thread_rng()).unwrap(), eta);
            } else {
                progress.blocked(REASONS.choose(&mut thread_rng()).unwrap(), eta);
            }
            thread_rng().gen_range(WORK_DELAY_MS, LONG_WORK_DELAY_MS)
        } else {
            thread_rng().gen_range(SHORT_DELAY_MS, WORK_DELAY_MS)
        };
        if thread_rng().gen_bool(0.01) {
            progress.init(Some(max.into()), UNITS.choose(&mut thread_rng()).copied())
        }
        if thread_rng().gen_bool(0.01) {
            progress.info(*INFO_MESSAGES.choose(&mut thread_rng()).unwrap());
        }
        if thread_rng().gen_bool(if changing_names { 0.5 } else { 0.01 }) {
            progress.set_name(WORK_NAMES.choose(&mut thread_rng()).unwrap().to_string());
        }
        smol::Timer::after(Duration::from_millis((delay_ms as f32 / speed) as u64)).await;
    }
    if thread_rng().gen_bool(0.95) {
        progress.done(*DONE_MESSAGES.choose(&mut thread_rng()).unwrap());
    } else {
        progress.fail(*FAIL_MESSAGES.choose(&mut thread_rng()).unwrap());
    }
}

async fn new_chunk_of_work(
    max: NestingLevel,
    tree: Tree,
    speed: f32,
    changing_names: bool,
) -> Result {
    let NestingLevel(max_level) = max;
    let mut progresses = Vec::new();
    let mut level_progress = tree.add_child(format!("level {} of {}", 1, max_level));
    let mut handles = Vec::new();

    for level in 0..max_level {
        // one-off ambient tasks
        let num_tasks = max_level as usize * 2;
        for id in 0..num_tasks {
            let handle = smol::Task::spawn(work_item(
                level_progress.add_child(format!(
                    "{} {}",
                    WORK_NAMES.choose(&mut thread_rng()).unwrap(),
                    id + 1
                )),
                speed,
                changing_names,
            ));
            handles.push(handle);

            smol::Timer::after(Duration::from_millis(
                (SPAWN_DELAY_MS as f32 / speed) as u64,
            ))
            .await;
        }
        if level + 1 != max_level {
            let tmp = level_progress.add_child(format!("Level {}", level + 1));
            progresses.push(level_progress);
            level_progress = tmp;
        }
    }

    progresses.push(level_progress);
    for handle in handles.into_iter() {
        handle.await;
    }

    Ok(())
}

enum Direction {
    Shrink,
    Grow,
}

fn generate_statistics() -> Vec<Line> {
    let mut lines = vec![
        Line::Text("You can put here what you want".into()),
        Line::Text("as long as it fits one line".into()),
        Line::Text("until a certain limit is reached".into()),
        Line::Text("which is when truncation happens".into()),
        Line::Text("这是中文的一些文字。".into()),
        Line::Text("鹅、鹅、鹅 曲项向天歌 白毛浮绿水 红掌拨清波".into()),
        Line::Text("床前明月光, 疑是地上霜。举头望明月，低头思故乡。".into()),
        Line::Text("锄禾日当午，汗滴禾下土。谁知盘中餐，粒粒皆辛苦。".into()),
        Line::Text("春眠不觉晓，处处闻啼鸟。夜来风雨声，花落知多少".into()),
        Line::Text("煮豆燃豆萁，豆在釜中泣。本自同根生，相煎何太急".into()),
        Line::Text(
            "and this line is without any doubt very very long and it really doesn't want to stop"
                .into(),
        ),
    ];
    lines.shuffle(&mut thread_rng());
    lines.insert(0, Line::Title("Hello World".into()));

    lines.extend(vec![
        Line::Title("Statistics".into()),
        Line::Text(format!(
            "lines of unsafe code: {}",
            thread_rng().gen_range(0usize, 1_000_000)
        )),
        Line::Text(format!(
            "wasted space in crates: {} Kb",
            thread_rng().gen_range(100usize, 1_000_000)
        )),
        Line::Text(format!(
            "unused dependencies: {} crates",
            thread_rng().gen_range(100usize, 1_000)
        )),
        Line::Text(format!(
            "average #dependencies: {} crates",
            thread_rng().gen_range(0usize, 500)
        )),
        Line::Text(format!(
            "bloat in code: {} Kb",
            thread_rng().gen_range(100usize, 5_000)
        )),
    ]);
    lines
}

fn window_resize_stream(animate: bool) -> impl futures_core::Stream<Item = Event> {
    let mut offset_xy = (0u16, 0u16);
    let mut direction = Direction::Shrink;
    if !animate {
        return futures_util::stream::pending().boxed();
    }

    ticker(Duration::from_millis(100))
        .map(move |_| {
            let (width, height) = crosstermion::crossterm::terminal::size().unwrap_or((30, 30));
            let (ref mut ofs_x, ref mut ofs_y) = offset_xy;
            let min_size = 2;
            match direction {
                Direction::Shrink => {
                    *ofs_x = ofs_x
                        .saturating_add((1_f32 * (width as f32 / height as f32)).ceil() as u16);
                    *ofs_y = ofs_y
                        .saturating_add((1_f32 * (height as f32 / width as f32)).ceil() as u16);
                }
                Direction::Grow => {
                    *ofs_x = ofs_x
                        .saturating_sub((1_f32 * (width as f32 / height as f32)).ceil() as u16);
                    *ofs_y = ofs_y
                        .saturating_sub((1_f32 * (height as f32 / width as f32)).ceil() as u16);
                }
            }
            let bound = tui::tui_export::layout::Rect {
                x: 0,
                y: 0,
                width: width.saturating_sub(*ofs_x).max(min_size),
                height: height.saturating_sub(*ofs_y).max(min_size),
            };
            if bound.area() <= min_size * min_size || bound.area() == width * height {
                direction = match direction {
                    Direction::Grow => Direction::Shrink,
                    Direction::Shrink => Direction::Grow,
                };
            }
            Event::SetWindowSize(bound)
        })
        .boxed()
}

struct NestingLevel(u8);
type Result = std::result::Result<(), Box<dyn Error + Send>>;

mod arg {
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

        /// multiplies the speed at which tasks seem to be running. Driving this down makes the TUI easier on the eyes
        /// Defaults to 1.0. A valud of 0.5 halves the speed.
        #[argh(option, short = 's', default = "1.0")]
        pub speed_multitplier: f32,

        /// if set (default: false), we will stop running the TUI once there the list of drawable progress items is empty.
        #[argh(switch)]
        pub stop_if_empty_progress: bool,

        /// set the renderer to use, defaults to "tui", and furthermore allows "line" and "log".
        ///
        /// If set ot "log", there will only be logging. Set 'RUST_LOG=info' before running the program to see them.
        #[argh(option)]
        pub renderer: Option<String>,
    }
}

use futures_util::{future::join_all, future::Either, FutureExt, StreamExt};
use prodash::{
    tree::{Item, Key},
    tui::{self, ticker, Event, Interrupt, Line},
    Tree,
};
use rand::prelude::*;
use std::{error::Error, ops::Add, time::Duration, time::SystemTime};

const WORK_STEPS_NEEDED_FOR_UNBOUNDED_TASK: u8 = 100;
const UNITS: &[&str] = &["Mb", "kb", "items", "files"];
const REASONS: &[&str] = &[
    "due to star alignment",
    "IO takes time",
    "仪表板演示",
    "just because",
];
const TITLES: &[&str] = &[" Dashboard Demo ", " 仪表板演示 "];
const WORK_NAMES: &[&str] = &[
    "Downloading Crate",
    "下载板条箱",
    "Running 'cargo geiger'",
    "运行程序 'cargo geiger'",
    "Counting lines of code",
    "计数代码行",
    "Checking for unused dependencies",
    "检查未使用的依赖项",
    "Checking for crate-bloat",
    "检查板条箱膨胀",
    "Generating report",
    "生成报告",
];
const DONE_MESSAGES: &[&str] = &[
    "Yeeeehaa! Finally!!",
    "呀！ 最后！",
    "It feels good to be done!",
    "感觉好极了！",
    "Told you so!!",
    "告诉过你了！",
];
const FAIL_MESSAGES: &[&str] = &[
    "That didn't seem to work!",
    "那似乎没有用！",
    "Oh my… I failed you 😞",
    "哦，我…我让你失败😞",
    "This didn't end well…",
    "结局不好…",
];
const INFO_MESSAGES: &[&str] = &[
    "Making good progress!",
    "进展良好！",
    "Humming along…",
    "嗡嗡作响…",
    "It will be done soooooon…",
    "会很快完成的……",
];
const SHORT_DELAY_MS: u64 = 50;
const WORK_DELAY_MS: u64 = 100;
const LONG_WORK_DELAY_MS: u64 = 2000;
const SPAWN_DELAY_MS: u64 = 200;
const CHANCE_TO_BLOCK_PER_STEP: f64 = 1.0 / 100.0;
const CHANCE_TO_SHOW_ETA: f64 = 0.5;
