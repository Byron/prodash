use crate::progress::StepShared;
use crate::{messages::MessageLevel, Progress, Unit};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// A [`Progress`] implementation which displays progress as it happens without the use of a renderer.
///
/// Note that this incurs considerable performance cost as each progress calls ends up getting the system time
/// to see if progress information should actually be emitted.
pub struct Log {
    name: String,
    max: Option<usize>,
    unit: Option<Unit>,
    step: usize,
    current_level: usize,
    max_level: usize,
    trigger: Arc<AtomicBool>,
}

const EMIT_LOG_EVERY_S: f32 = 0.5;
const SEP: &str = "::";

impl Log {
    /// Create a new instance from `name` while displaying progress information only up to `max_level`.
    pub fn new(name: impl Into<String>, max_level: Option<usize>) -> Self {
        let trigger = Arc::new(AtomicBool::new(true));
        std::thread::spawn({
            let duration = Duration::from_secs_f32(EMIT_LOG_EVERY_S);
            let trigger = Arc::downgrade(&trigger);
            move || {
                while let Some(t) = trigger.upgrade() {
                    t.store(true, Ordering::Relaxed);
                    std::thread::sleep(duration);
                }
            }
        });
        Log {
            name: name.into(),
            current_level: 0,
            max_level: max_level.unwrap_or(usize::MAX),
            max: None,
            step: 0,
            unit: None,
            trigger,
        }
    }
}

impl Progress for Log {
    type SubProgress = Log;

    fn add_child(&mut self, name: impl Into<String>) -> Self::SubProgress {
        Log {
            name: format!("{}{}{}", self.name, SEP, Into::<String>::into(name)),
            current_level: self.current_level + 1,
            max_level: self.max_level,
            step: 0,
            max: None,
            unit: None,
            trigger: Arc::clone(&self.trigger),
        }
    }

    fn init(&mut self, max: Option<usize>, unit: Option<Unit>) {
        self.max = max;
        self.unit = unit;
    }

    fn set(&mut self, step: usize) {
        self.step = step;
        if self.current_level > self.max_level {
            return;
        }
        if self.trigger.swap(false, Ordering::Relaxed) {
            match (self.max, &self.unit) {
                (max, Some(unit)) => log::info!("{} → {}", self.name, unit.display(step, max, None)),
                (Some(max), None) => log::info!("{} → {} / {}", self.name, step, max),
                (None, None) => log::info!("{} → {}", self.name, step),
            }
        }
    }

    fn unit(&self) -> Option<Unit> {
        self.unit.clone()
    }

    fn max(&self) -> Option<usize> {
        self.max
    }

    fn step(&self) -> usize {
        self.step
    }

    fn inc_by(&mut self, step: usize) {
        self.set(self.step + step)
    }

    fn set_name(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.name = self
            .name
            .split("::")
            .next()
            .map(|parent| format!("{}{}{}", parent.to_owned(), SEP, name))
            .unwrap_or(name);
    }

    fn name(&self) -> Option<String> {
        self.name.split(SEP).nth(1).map(ToOwned::to_owned)
    }

    fn message(&mut self, level: MessageLevel, message: impl Into<String>) {
        let message: String = message.into();
        match level {
            MessageLevel::Info => log::info!("ℹ{} → {}", self.name, message),
            MessageLevel::Failure => log::error!("𐄂{} → {}", self.name, message),
            MessageLevel::Success => log::info!("✓{} → {}", self.name, message),
        }
    }

    fn counter(&self) -> Option<StepShared> {
        None
    }
}
