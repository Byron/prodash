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

use crate::{messages::MessageLevel, Progress};

pub struct Discard;

impl Progress for Discard {
    type SubProgress = Discard;

    fn add_child(&mut self, _name: impl Into<String>) -> Self::SubProgress {
        Discard
    }

    fn init(&mut self, _max: Option<usize>, _unit: Option<Unit>) {}

    fn set(&mut self, _step: usize) {}

    fn inc_by(&mut self, _step: usize) {}

    fn message(&mut self, _level: MessageLevel, _message: impl Into<String>) {}
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Progress for Either<L, R>
where
    L: Progress,
    R: Progress,
{
    type SubProgress = Either<L::SubProgress, R::SubProgress>;

    fn add_child(&mut self, name: impl Into<String>) -> Self::SubProgress {
        match self {
            Either::Left(l) => Either::Left(l.add_child(name)),
            Either::Right(r) => Either::Right(r.add_child(name)),
        }
    }

    fn init(&mut self, max: Option<usize>, unit: Option<Unit>) {
        match self {
            Either::Left(l) => l.init(max, unit),
            Either::Right(r) => r.init(max, unit),
        }
    }

    fn set(&mut self, step: usize) {
        match self {
            Either::Left(l) => l.set(step),
            Either::Right(r) => r.set(step),
        }
    }

    fn inc_by(&mut self, step: usize) {
        match self {
            Either::Left(l) => l.inc_by(step),
            Either::Right(r) => r.inc_by(step),
        }
    }

    fn message(&mut self, level: MessageLevel, message: impl Into<String>) {
        match self {
            Either::Left(l) => l.message(level, message),
            Either::Right(r) => r.message(level, message),
        }
    }
}

pub struct DoOrDiscard<T>(Either<T, Discard>);

impl<T> From<Option<T>> for DoOrDiscard<T>
where
    T: Progress,
{
    fn from(p: Option<T>) -> Self {
        match p {
            Some(p) => DoOrDiscard(Either::Left(p)),
            None => DoOrDiscard(Either::Right(Discard)),
        }
    }
}
impl<T> DoOrDiscard<T> {
    pub fn into_inner(self) -> Option<T> {
        match self {
            DoOrDiscard(Either::Left(p)) => Some(p),
            DoOrDiscard(Either::Right(_)) => None,
        }
    }
}

impl<T> Progress for DoOrDiscard<T>
where
    T: Progress,
{
    type SubProgress = DoOrDiscard<T::SubProgress>;

    fn add_child(&mut self, name: impl Into<String>) -> Self::SubProgress {
        DoOrDiscard(self.0.add_child(name))
    }

    fn init(&mut self, max: Option<usize>, unit: Option<Unit>) {
        self.0.init(max, unit)
    }

    fn set(&mut self, step: usize) {
        self.0.set(step)
    }

    fn inc_by(&mut self, step: usize) {
        self.0.inc_by(step)
    }

    fn message(&mut self, level: MessageLevel, message: impl Into<String>) {
        self.0.message(level, message)
    }
}
