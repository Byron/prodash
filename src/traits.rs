use crate::{messages::MessageLevel, progress, Unit};
use std::time::Instant;

/// A trait for describing hierarchical process.
pub trait Progress: Send + 'static {
    /// The type of progress returned by [`add_child()`][Progress::add_child()].
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
    fn init(&mut self, max: Option<progress::Step>, unit: Option<Unit>);

    /// Set the current progress to the given `step`. The cost of this call is negligible,
    /// making manual throttling *not* necessary.
    ///
    /// **Note**: that this call has no effect unless `init(…)` was called before.
    fn set(&mut self, step: progress::Step);

    /// Returns the (cloned) unit associated with this Progress
    fn unit(&self) -> Option<Unit> {
        None
    }

    /// Returns the maximum about of items we expect, as provided with the `init(…)` call
    fn max(&self) -> Option<progress::Step> {
        None
    }

    /// Returns the current step, as controlled by `inc*(…)` calls
    fn step(&self) -> progress::Step;

    /// Increment the current progress to the given `step`.
    /// The cost of this call is negligible, making manual throttling *not* necessary.
    fn inc_by(&mut self, step: progress::Step);

    /// Increment the current progress to the given 1. The cost of this call is negligible,
    /// making manual throttling *not* necessary.
    fn inc(&mut self) {
        self.inc_by(1)
    }

    /// Set the name of the instance, altering the value given when crating it with `add_child(…)`
    /// The progress is allowed to discard it.
    fn set_name(&mut self, name: impl Into<String>);

    /// Get the name of the instance as given when creating it with `add_child(…)`
    /// The progress is allowed to not be named, thus there is no guarantee that a previously set names 'sticks'.
    fn name(&self) -> Option<String>;

    /// Create a `message` of the given `level` and store it with the progress tree.
    ///
    /// Use this to provide additional, human-readable information about the progress
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
    fn show_throughput(&mut self, start: Instant) {
        let step = self.step();
        match self.unit() {
            Some(unit) => self.show_throughput_with(start, step, unit),
            None => {
                let elapsed = start.elapsed().as_secs_f32();
                let steps_per_second = (step as f32 / elapsed) as progress::Step;
                self.info(format!(
                    "done {} items in {:.02}s ({} items/s)",
                    step, elapsed, steps_per_second
                ))
            }
        };
    }

    /// A shorthand to print throughput information, with the given step and unit
    fn show_throughput_with(&mut self, start: Instant, step: progress::Step, unit: Unit) {
        use std::fmt::Write;
        let elapsed = start.elapsed().as_secs_f32();
        let steps_per_second = (step as f32 / elapsed) as progress::Step;
        let mut buf = String::with_capacity(128);
        let unit = unit.as_display_value();
        let push_unit = |buf: &mut String| {
            buf.push(' ');
            let len_before_unit = buf.len();
            unit.display_unit(buf, step).ok();
            if buf.len() == len_before_unit {
                buf.pop();
            }
        };

        buf.push_str("done ");
        unit.display_current_value(&mut buf, step, None).ok();
        push_unit(&mut buf);

        buf.write_fmt(format_args!(" in {:.02}s (", elapsed)).ok();
        unit.display_current_value(&mut buf, steps_per_second, None).ok();
        push_unit(&mut buf);
        buf.push_str("/s)");

        self.info(buf);
    }
}

use crate::messages::{Envelope, MessageCopyState};

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

    /// Create a raw `message` and store it with the progress tree.
    ///
    /// Use this to render additional unclassified output about the progress made.
    fn message_raw(&mut self, message: impl Into<String>);

    /// Copy all messages from the internal ring buffer into the given `out`
    /// vector. Messages are ordered from oldest to newest.
    fn copy_messages(&self, out: &mut Vec<Envelope>);

    /// Copy only new messages from the internal ring buffer into the given `out`
    /// vector. Messages are ordered from oldest to newest.
    fn copy_new_messages(&self, out: &mut Vec<Envelope>, prev: Option<MessageCopyState>) -> MessageCopyState;
}
