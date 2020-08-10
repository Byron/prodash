use crate::{messages::MessageLevel, Progress, Unit};

pub struct Discard;

impl Progress for Discard {
    type SubProgress = Discard;

    fn add_child(&mut self, _name: impl Into<String>) -> Self::SubProgress {
        Discard
    }

    fn init(&mut self, _max: Option<usize>, _unit: Option<Unit>) {}

    fn set(&mut self, _step: usize) {}

    fn step(&self) -> usize {
        0
    }

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

impl<T: Progress> DoOrDiscard<T> {
    pub fn into_inner(self) -> Option<T> {
        match self {
            DoOrDiscard(Either::Left(p)) => Some(p),
            DoOrDiscard(Either::Right(_)) => None,
        }
    }

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

    fn step(&self) -> usize {
        self.0.step()
    }

    fn inc_by(&mut self, step: usize) {
        self.0.inc_by(step)
    }

    fn message(&mut self, level: MessageLevel, message: impl Into<String>) {
        self.0.message(level, message)
    }
}
