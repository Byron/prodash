use crate::{messages::MessageLevel, progress, Unit};
use std::time::Instant;

pub trait Progress {
    type SubProgress: Progress;

    /// Adds a new child, whose parent is this instance, with the given name.
    ///
    /// This will make the child progress to appear contained in the parent progress.
    fn add_child(&mut self, name: impl Into<String>) -> Self::SubProgress;

    /// Initialize the Item for receiving progress information.
    ///
    /// If `max` is `Some(…)`, it will be treated as upper bound. When progress is [set(…)](./struct.Item.html#method.set)
    /// it should not exceed the given maximum.
    /// If `max` is `None`, the progress is unbounded. Use this if the amount of work cannot accurately
    /// be determined in advance.
    ///
    /// If `unit` is `Some(…)`, it is used for display purposes only. See `prodash::Unit` for more information.
    ///
    /// If this method is never called, this `Progress` instance will serve as organizational unit, useful to add more structure
    /// to the progress tree (e.g. a headline).
    ///
    /// **Note** that this method can be called multiple times, changing the bounded-ness and unit at will.
    fn init(&mut self, max: Option<usize>, unit: Option<Unit>);

    /// Set the current progress to the given `step`. The cost of this call is negligible,
    /// making manual throttling *not* necessary.
    ///
    /// **Note**: that this call has no effect unless `init(…)` was called before.
    fn set(&mut self, step: usize);

    /// Increment the current progress to the given `step`. The cost of this call is negligible,
    /// making manual throttling *not* necessary.
    ///
    /// **Note**: that this call has no effect unless `init(…)` was called before.
    fn inc_by(&mut self, step: usize);

    /// Increment the current progress to the given 1. The cost of this call is negligible,
    /// making manual throttling *not* necessary.
    ///
    /// **Note**: that this call has no effect unless `init(…)` was called before.
    fn inc(&mut self) {
        self.inc_by(1)
    }

    /// Create a `message` of the given `level` and store it with the progress tree.
    ///
    /// Use this to provide additional,human-readable information about the progress
    /// made, including indicating success or failure.
    fn message(&mut self, level: MessageLevel, message: impl Into<String>);

    /// Create a message providing additional information about the progress thus far.
    fn info(&mut self, message: impl Into<String>) {
        self.message(MessageLevel::Info, message)
    }
    /// Create a message indicating the task is done successfully
    fn done(&mut self, message: impl Into<String>) {
        self.message(MessageLevel::Success, message)
    }
    /// Create a message indicating the task failed
    fn fail(&mut self, message: impl Into<String>) {
        self.message(MessageLevel::Failure, message)
    }
    /// A shorthand to print throughput information
    fn show_throughput(&mut self, start: Instant, total_items: usize, item_name: &str) {
        let elapsed = start.elapsed().as_secs_f32();
        self.info(format!(
            "done {} {} in {:.02}s ({} {}/s)",
            total_items,
            item_name,
            elapsed,
            total_items as f32 / elapsed,
            item_name
        ));
    }
}

use crate::messages::{Message, MessageCopyState};

/// The top level of a progress task hiearchy, with `progress::Task`s identified with `progress::Key`s
pub trait Root {
    /// Returns the maximum amount of messages we can keep before overwriting older ones.
    fn messages_capacity(&self) -> usize;

    /// Returns the current amount of tasks underneath the root, transitively.
    /// **Note** that this is at most a guess as tasks can be added and removed in parallel.
    fn num_tasks(&self) -> usize;

    /// Copy the entire progress tree into the given `out` vector, so that
    /// it can be traversed from beginning to end in order of hierarchy.
    /// The `out` vec will be cleared automatically.
    fn sorted_snapshot(&self, out: &mut Vec<(progress::Key, progress::Task)>);

    /// Copy all messages from the internal ring buffer into the given `out`
    /// vector. Messages are ordered from oldest to newest.
    fn copy_messages(&self, out: &mut Vec<Message>);

    /// Copy only new messages from the internal ring buffer into the given `out`
    /// vector. Messages are ordered from oldest to newest.
    fn copy_new_messages(&self, out: &mut Vec<Message>, prev: Option<MessageCopyState>) -> MessageCopyState;
}
