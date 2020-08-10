use crate::unit::Unit;
use std::time::SystemTime;

/// The amount of steps a progress can make
pub type Step = usize;

/// Indicate whether a progress can or cannot be made.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum State {
    /// Indicates a task is blocked and cannot indicate progress, optionally until the
    /// given time. The task cannot easily be interrupted.
    Blocked(&'static str, Option<SystemTime>),
    /// Indicates a task cannot indicate progress, optionally until the
    /// given time. The task can be interrupted.
    Halted(&'static str, Option<SystemTime>),
    /// The task is running
    Running,
}

impl Default for State {
    fn default() -> Self {
        State::Running
    }
}

/// Progress associated with some item in the progress tree.
#[derive(Clone, Default, Debug)]
pub struct Value {
    /// The amount of progress currently made
    pub step: Step,
    /// The step at which no further progress has to be made.
    ///
    /// If unset, the progress is unbounded.
    pub done_at: Option<Step>,
    /// The unit associated with the progress.
    pub unit: Option<Unit>,
    /// Whether progress can be made or not
    pub state: State,
}

impl Value {
    /// Returns a number between `Some(0.0)` and `Some(1.0)`, or `None` if the progress is unbounded.
    ///
    /// A task half done would return `Some(0.5)`.
    pub fn fraction(&self) -> Option<f32> {
        self.done_at.map(|done_at| self.step as f32 / done_at as f32)
    }
}

/// The value associated with a spot in the hierarchy.
#[derive(Clone, Default, Debug)]
pub struct Task {
    /// The name of the `Item` or task.
    pub name: String,
    /// The progress itself, unless this value belongs to an `Item` serving as organizational unit.
    pub progress: Option<Value>,
}
