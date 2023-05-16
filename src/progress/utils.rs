use crate::{messages::MessageLevel, progress::Id, Progress, Unit};

/// An implementation of [`Progress`] which discards all calls.
pub struct Discard;

impl Progress for Discard {
    type SubProgress = Discard;

    fn add_child(&mut self, _name: impl Into<String>) -> Self::SubProgress {
        Discard
    }

    fn add_child_with_id(&mut self, _name: impl Into<String>, _id: Id) -> Self::SubProgress {
        Discard
    }

    fn init(&mut self, _max: Option<usize>, _unit: Option<Unit>) {}

    fn set(&mut self, _step: usize) {}

    fn set_max(&mut self, _max: Option<Step>) -> Option<Step> {
        None
    }

    fn step(&self) -> usize {
        0
    }

    fn inc_by(&mut self, _step: usize) {}

    fn set_name(&mut self, _name: impl Into<String>) {}

    fn name(&self) -> Option<String> {
        None
    }

    fn id(&self) -> Id {
        crate::progress::UNKNOWN
    }

    fn message(&self, _level: MessageLevel, _message: impl Into<String>) {}

    fn counter(&self) -> Option<StepShared> {
        None
    }
}

/// An implementation of [`Progress`] showing either one or the other implementation.
///
/// Useful in conjunction with [`Discard`] and a working implementation, making it as a form of `Option<Progress>` which
/// can be passed to methods requiring `impl Progress`.
/// See [`DoOrDiscard`] for an incarnation of this.
#[allow(missing_docs)]
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

    fn add_child_with_id(&mut self, name: impl Into<String>, id: Id) -> Self::SubProgress {
        match self {
            Either::Left(l) => Either::Left(l.add_child_with_id(name, id)),
            Either::Right(r) => Either::Right(r.add_child_with_id(name, id)),
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

    fn unit(&self) -> Option<Unit> {
        match self {
            Either::Left(l) => l.unit(),
            Either::Right(r) => r.unit(),
        }
    }

    fn max(&self) -> Option<usize> {
        match self {
            Either::Left(l) => l.max(),
            Either::Right(r) => r.max(),
        }
    }

    fn set_max(&mut self, max: Option<Step>) -> Option<Step> {
        match self {
            Either::Left(l) => l.set_max(max),
            Either::Right(r) => r.set_max(max),
        }
    }

    fn step(&self) -> usize {
        match self {
            Either::Left(l) => l.step(),
            Either::Right(r) => r.step(),
        }
    }

    fn inc_by(&mut self, step: usize) {
        match self {
            Either::Left(l) => l.inc_by(step),
            Either::Right(r) => r.inc_by(step),
        }
    }

    fn set_name(&mut self, name: impl Into<String>) {
        match self {
            Either::Left(l) => l.set_name(name),
            Either::Right(r) => r.set_name(name),
        }
    }

    fn name(&self) -> Option<String> {
        match self {
            Either::Left(l) => l.name(),
            Either::Right(r) => r.name(),
        }
    }

    fn id(&self) -> Id {
        todo!()
    }

    fn message(&self, level: MessageLevel, message: impl Into<String>) {
        match self {
            Either::Left(l) => l.message(level, message),
            Either::Right(r) => r.message(level, message),
        }
    }

    fn counter(&self) -> Option<StepShared> {
        match self {
            Either::Left(l) => l.counter(),
            Either::Right(r) => r.counter(),
        }
    }
}

/// An implementation of `Progress` which can be created easily from `Option<impl Progress>`.
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

impl<T: Progress> DoOrDiscard<T> {
    /// Obtain either the original [`Progress`] implementation or `None`.
    pub fn into_inner(self) -> Option<T> {
        match self {
            DoOrDiscard(Either::Left(p)) => Some(p),
            DoOrDiscard(Either::Right(_)) => None,
        }
    }

    /// Take out the implementation of [`Progress`] and replace it with [`Discard`].
    pub fn take(&mut self) -> Option<T> {
        let this = std::mem::replace(self, DoOrDiscard::from(None));
        match this {
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

    fn add_child_with_id(&mut self, name: impl Into<String>, id: Id) -> Self::SubProgress {
        DoOrDiscard(self.0.add_child_with_id(name, id))
    }

    fn init(&mut self, max: Option<usize>, unit: Option<Unit>) {
        self.0.init(max, unit)
    }

    fn set(&mut self, step: usize) {
        self.0.set(step)
    }

    fn unit(&self) -> Option<Unit> {
        self.0.unit()
    }

    fn max(&self) -> Option<usize> {
        self.0.max()
    }

    fn set_max(&mut self, max: Option<Step>) -> Option<Step> {
        self.0.set_max(max)
    }

    fn step(&self) -> usize {
        self.0.step()
    }

    fn inc_by(&mut self, step: usize) {
        self.0.inc_by(step)
    }

    fn set_name(&mut self, name: impl Into<String>) {
        self.0.set_name(name);
    }

    fn name(&self) -> Option<String> {
        self.0.name()
    }

    fn id(&self) -> Id {
        self.0.id()
    }

    fn message(&self, level: MessageLevel, message: impl Into<String>) {
        self.0.message(level, message)
    }

    fn counter(&self) -> Option<StepShared> {
        self.0.counter()
    }
}

use std::time::Instant;

use crate::progress::{Step, StepShared};

/// Emit a message with throughput information when the instance is dropped.
pub struct ThroughputOnDrop<T: Progress>(T, Instant);

impl<T: Progress> ThroughputOnDrop<T> {
    /// Create a new instance by providing the `inner` [`Progress`] implementation.
    pub fn new(inner: T) -> Self {
        ThroughputOnDrop(inner, Instant::now())
    }
}

impl<T: Progress> Progress for ThroughputOnDrop<T> {
    type SubProgress = T::SubProgress;

    fn add_child(&mut self, name: impl Into<String>) -> Self::SubProgress {
        self.0.add_child(name)
    }

    fn add_child_with_id(&mut self, name: impl Into<String>, id: Id) -> Self::SubProgress {
        self.0.add_child_with_id(name, id)
    }

    fn init(&mut self, max: Option<usize>, unit: Option<Unit>) {
        self.0.init(max, unit)
    }

    fn set(&mut self, step: usize) {
        self.0.set(step)
    }

    fn unit(&self) -> Option<Unit> {
        self.0.unit()
    }

    fn max(&self) -> Option<usize> {
        self.0.max()
    }

    fn set_max(&mut self, max: Option<Step>) -> Option<Step> {
        self.0.set_max(max)
    }

    fn step(&self) -> usize {
        self.0.step()
    }

    fn inc_by(&mut self, step: usize) {
        self.0.inc_by(step)
    }

    fn set_name(&mut self, name: impl Into<String>) {
        self.0.set_name(name)
    }

    fn name(&self) -> Option<String> {
        self.0.name()
    }

    fn id(&self) -> Id {
        self.0.id()
    }

    fn message(&self, level: MessageLevel, message: impl Into<String>) {
        self.0.message(level, message)
    }

    fn counter(&self) -> Option<StepShared> {
        self.0.counter()
    }
}

impl<T: Progress> Drop for ThroughputOnDrop<T> {
    fn drop(&mut self) {
        self.0.show_throughput(self.1)
    }
}
