#![deny(unsafe_code)]

#[cfg(not(feature = "render-tui"))]
compile_error!("The `render-tui` feature must be set, along with the `render-tui-crossterm`");
#[cfg(not(any(feature = "render-tui-crossterm")))]
compile_error!("Please set the 'render-tui-crossterm' feature when using the 'render-tui'");

use std::{error::Error, sync::Arc};

use prodash::{tree::Root as Tree, unit};
use shared::args;

fn main() -> Result {
    env_logger::init();

    let args: args::Options = argh::from_env();
    let root = Tree::new();
    let renderer = args.renderer.clone().unwrap_or_else(|| "line".into());
    let handle = shared::launch_ambient_gui(root.clone(), &renderer, args, true).unwrap();
    let work = async move {
        let mut unblock = blocking::Unblock::new(());
        unblock.with_mut(move |_| work_for_a_long_time_blocking(root)).await
    };
    futures_lite::pin!(work);
    futures_lite::future::block_on(futures_util::future::select(handle, work));
    Ok(())
}

fn work_for_a_long_time_blocking(root: Arc<Tree>) {
    let bytes = root.add_child_with_id("download unknown", *b"DLUK");
    bytes.init(
        None,
        Some(unit::dynamic_and_mode(
            unit::Bytes,
            unit::display::Mode::with_throughput(),
        )),
    );
    let bytes_max = root.add_child_with_id("download", *b"DLKN");
    bytes_max.init(
        Some(100_000_000),
        Some(unit::dynamic_and_mode(
            unit::Bytes,
            unit::display::Mode::with_percentage().and_throughput(),
        )),
    );

    let duration = root.add_child_with_id("duration unknown", *b"DRUK");
    duration.init(None, Some(unit::dynamic(unit::Duration)));
    let duration_max = root.add_child_with_id("duration", *b"DRKN");
    duration_max.init(
        Some(60 * 60 * 24),
        Some(unit::dynamic_and_mode(
            unit::Duration,
            unit::display::Mode::with_percentage().show_before_value(),
        )),
    );

    fn formatter(decimals: usize) -> unit::human::Formatter {
        let mut f = unit::human::Formatter::new();
        f.with_decimals(decimals);
        f
    }
    let human_count = root.add_child_with_id("item count unknown", *b"ITUK");
    human_count.init(
        None,
        Some(unit::dynamic_and_mode(
            unit::Human::new(formatter(0), "items"),
            unit::display::Mode::with_throughput(),
        )),
    );
    let human_count_max = root.add_child_with_id("item count", *b"ITKN");
    human_count_max.init(
        Some(7_542_241),
        Some(unit::dynamic_and_mode(
            unit::Human::new(formatter(2), "items"),
            unit::display::Mode::with_percentage().and_throughput(),
        )),
    );

    let steps = root.add_child_with_id("steps to take unknown", *b"STUK");
    steps.init(
        None,
        Some(unit::dynamic_and_mode(
            unit::Range::new("steps"),
            unit::display::Mode::with_throughput(),
        )),
    );
    let steps_max = root.add_child_with_id("steps to take", *b"STKN");
    steps_max.init(
        Some(100),
        Some(unit::dynamic_and_mode(
            unit::Range::new("steps"),
            unit::display::Mode::with_percentage().and_throughput(),
        )),
    );

    let steps_per_second = 10;
    for step in 0.. {
        bytes_max.inc_by(1_459_121);
        bytes.inc_by(23_212_159);

        duration.inc();
        duration_max.inc_by(60);

        human_count.inc_by(4);
        human_count_max.inc_by(40274 / steps_per_second);

        if step % steps_per_second == 0 {
            steps.inc();
            steps_max.inc();
        }
        std::thread::sleep(std::time::Duration::from_millis((1000 / steps_per_second) as u64));
    }
}

type Result = std::result::Result<(), Box<dyn Error + Send + 'static>>;

mod shared;
