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
