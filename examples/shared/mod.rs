use crate::Direction;
use futures_util::{future::FutureExt, stream::StreamExt};
use prodash::{
    line, tui,
    tui::Line,
    tui::{ticker, Event, Interrupt},
    Tree,
};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{ops::RangeInclusive, time::Duration};

pub mod args;
pub mod smol;

const TITLES: &[&str] = &[" Dashboard Demo ", " 仪表板演示 "];

pub fn launch_ambient_gui(
    progress: Tree,
    renderer: &str,
    args: args::Options,
) -> std::result::Result<smol::Task<()>, std::io::Error> {
    let mut ticks: usize = 0;
    let mut interruptible = true;
    let render_fut = match renderer {
        "line" => async move {
            let output_is_terminal = atty::is(atty::Stream::Stdout);
            let mut handle = line::render(
                std::io::stderr(),
                progress,
                line::Options {
                    output_is_terminal,
                    #[cfg(feature = "ctrlc")]
                    hide_cursor: true,
                    #[cfg(not(feature = "ctrlc"))]
                    hide_cursor: false,
                    terminal_dimensions: crosstermion::terminal::size()
                        .ok()
                        .map(|(w, h)| args.line_column_count.map(|width| (width, h)).unwrap_or((w, h)))
                        .unwrap_or((80, 20)),
                    timestamp: args.line_timestamp,
                    colored: !args.no_line_color && output_is_terminal && crosstermion::color::allowed(),
                    level_filter: Some(RangeInclusive::new(
                        args.line_start.unwrap_or(1),
                        args.line_end.unwrap_or(2),
                    )),
                    initial_delay: args.line_initial_delay.map(Duration::from_secs_f32),
                    frames_per_second: args.fps,
                    keep_running_if_progress_is_empty: true,
                },
            );
            handle.disconnect();
            blocking::unblock!(handle.wait());
        }
        .boxed(),
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
        )?
        .boxed(),
        _ => panic!("Unknown renderer: '{}'", renderer),
    };
    let handle = smol::Task::spawn(render_fut.map(|_| ()));
    Ok(handle)
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
        Line::Text("and this line is without any doubt very very long and it really doesn't want to stop".into()),
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
        Line::Text(format!("bloat in code: {} Kb", thread_rng().gen_range(100usize, 5_000))),
    ]);
    lines
}

fn window_resize_stream(animate: bool) -> impl futures_core::Stream<Item = Event> {
    let mut offset_xy = (0u16, 0u16);
    let mut direction = Direction::Shrink;
    if !animate {
        return futures_lite::stream::pending().boxed();
    }

    ticker(Duration::from_millis(100))
        .map(move |_| {
            let (width, height) = crosstermion::terminal::size().unwrap_or((30, 30));
            let (ref mut ofs_x, ref mut ofs_y) = offset_xy;
            let min_size = 2;
            match direction {
                Direction::Shrink => {
                    *ofs_x = ofs_x.saturating_add((1_f32 * (width as f32 / height as f32)).ceil() as u16);
                    *ofs_y = ofs_y.saturating_add((1_f32 * (height as f32 / width as f32)).ceil() as u16);
                }
                Direction::Grow => {
                    *ofs_x = ofs_x.saturating_sub((1_f32 * (width as f32 / height as f32)).ceil() as u16);
                    *ofs_y = ofs_y.saturating_sub((1_f32 * (height as f32 / width as f32)).ceil() as u16);
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
