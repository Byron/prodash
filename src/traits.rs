use std::time::Instant;

use crate::{messages::MessageLevel, progress, progress::Id, Unit};

/// A trait for describing hierarchical progress.
pub trait Progress: Send + Sync {
    /// The type of progress returned by [`add_child()`][Progress::add_child()].
    type SubProgress: Progress;

    /// Adds a new child, whose parent is this instance, with the given `name`.
    ///
    /// This will make the child progress to appear contained in the parent progress.
    /// Note that such progress does not have a stable identifier, which can be added
    /// with [`add_child_with_id()`][Progress::add_child_with_id()] if desired.
    fn add_child(&mut self, name: impl Into<String>) -> Self::SubProgress;

    /// Adds a new child, whose parent is this instance, with the given `name` and `id`.
    ///
    /// This will make the child progress to appear contained in the parent progress, and it can be identified
    /// using `id`.
    fn add_child_with_id(&mut self, name: impl Into<String>, id: Id) -> Self::SubProgress;

    /// Initialize the Item for receiving progress information.
    ///
    /// If `max` is `Some(…)`, it will be treated as upper bound. When progress is [set(…)](./struct.Item.html#method.set)
    /// it should not exceed the given maximum.
    /// If `max` is `None`, the progress is unbounded. Use this if the amount of work cannot accurately
    /// be determined in advance.
    ///
    /// If `unit` is `Some(…)`, it is used for display purposes only. See `prodash::Unit` for more information.
    ///
    /// If both `unit` and `max` are `None`, the item will be reset to be equivalent to 'uninitialized'.
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

    /// Set the maximum value to `max` and return the old maximum value.
    fn set_max(&mut self, _max: Option<progress::Step>) -> Option<progress::Step> {
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

    /// Get a stable identifier for the progress instance.
    /// Note that it could be [unknown][crate::progress::UNKNOWN].
    fn id(&self) -> Id;

    /// Create a `message` of the given `level` and store it with the progress tree.
    ///
    /// Use this to provide additional,human-readable information about the progress
    /// made, including indicating success or failure.
    fn message(&self, level: MessageLevel, message: impl Into<String>);

    /// If available, return an atomic counter for direct access to the underlying state.
    ///
    /// This is useful if multiple threads want to access the same progress, without the need
    /// for provide each their own progress and aggregating the result.
    fn counter(&self) -> Option<StepShared> {
        None
    }

    /// Create a message providing additional information about the progress thus far.
    fn info(&self, message: impl Into<String>) {
        self.message(MessageLevel::Info, message)
    }
    /// Create a message indicating the task is done successfully
    fn done(&self, message: impl Into<String>) {
        self.message(MessageLevel::Success, message)
    }
    /// Create a message indicating the task failed
    fn fail(&self, message: impl Into<String>) {
        self.message(MessageLevel::Failure, message)
    }
    /// A shorthand to print throughput information
    fn show_throughput(&self, start: Instant) {
        let step = self.step();
        match self.unit() {
            Some(unit) => self.show_throughput_with(start, step, unit, MessageLevel::Info),
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

    /// A shorthand to print throughput information, with the given step and unit, and message level.
    fn show_throughput_with(&self, start: Instant, step: progress::Step, unit: Unit, level: MessageLevel) {
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

        self.message(level, buf);
    }
}

/// A trait for describing non-hierarchical progress.
///
/// It differs by not being able to add child progress dynamically, but in turn is object safe. It's recommended to
/// use this trait whenever there is no need to add child progress, at the leaf of a computation.
// NOTE: keep this in-sync with `Progress`.
pub trait RawProgress: Send + Sync {
    /// Initialize the Item for receiving progress information.
    ///
    /// If `max` is `Some(…)`, it will be treated as upper bound. When progress is [set(…)](./struct.Item.html#method.set)
    /// it should not exceed the given maximum.
    /// If `max` is `None`, the progress is unbounded. Use this if the amount of work cannot accurately
    /// be determined in advance.
    ///
    /// If `unit` is `Some(…)`, it is used for display purposes only. See `prodash::Unit` for more information.
    ///
    /// If both `unit` and `max` are `None`, the item will be reset to be equivalent to 'uninitialized'.
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

    /// Set the maximum value to `max` and return the old maximum value.
    fn set_max(&mut self, _max: Option<progress::Step>) -> Option<progress::Step> {
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
    fn set_name(&mut self, name: String);

    /// Get the name of the instance as given when creating it with `add_child(…)`
    /// The progress is allowed to not be named, thus there is no guarantee that a previously set names 'sticks'.
    fn name(&self) -> Option<String>;

    /// Get a stable identifier for the progress instance.
    /// Note that it could be [unknown][crate::progress::UNKNOWN].
    fn id(&self) -> Id;

    /// Create a `message` of the given `level` and store it with the progress tree.
    ///
    /// Use this to provide additional,human-readable information about the progress
    /// made, including indicating success or failure.
    fn message(&self, level: MessageLevel, message: String);

    /// If available, return an atomic counter for direct access to the underlying state.
    ///
    /// This is useful if multiple threads want to access the same progress, without the need
    /// for provide each their own progress and aggregating the result.
    fn counter(&self) -> Option<StepShared> {
        None
    }

    /// Create a message providing additional information about the progress thus far.
    fn info(&self, message: String) {
        self.message(MessageLevel::Info, message)
    }
    /// Create a message indicating the task is done successfully
    fn done(&self, message: String) {
        self.message(MessageLevel::Success, message)
    }
    /// Create a message indicating the task failed
    fn fail(&self, message: String) {
        self.message(MessageLevel::Failure, message)
    }
    /// A shorthand to print throughput information
    fn show_throughput(&self, start: Instant) {
        let step = self.step();
        match self.unit() {
            Some(unit) => self.show_throughput_with(start, step, unit, MessageLevel::Info),
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

    /// A shorthand to print throughput information, with the given step and unit, and message level.
    fn show_throughput_with(&self, start: Instant, step: progress::Step, unit: Unit, level: MessageLevel) {
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

        self.message(level, buf);
    }
}

use crate::{
    messages::{Message, MessageCopyState},
    progress::StepShared,
};

/// The top-level root as weak handle, which needs an upgrade to become a usable root.
///
/// If the underlying reference isn't present anymore, such upgrade will fail permanently.
pub trait WeakRoot {
    /// The type implementing the `Root` trait
    type Root: Root;

    /// Equivalent to `std::sync::Weak::upgrade()`.
    fn upgrade(&self) -> Option<Self::Root>;
}

/// The top level of a progress task hierarchy, with `progress::Task`s identified with `progress::Key`s
pub trait Root {
    /// The type implementing the `WeakRoot` trait
    type WeakRoot: WeakRoot;

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

    /// Similar to `Arc::downgrade()`
    fn downgrade(&self) -> Self::WeakRoot;
}

mod impls {
    use std::{
        ops::{Deref, DerefMut},
        time::Instant,
    };

    use crate::traits::RawProgress;
    use crate::{
        messages::MessageLevel,
        progress::{Id, Step, StepShared},
        Progress, Unit,
    };

    impl<T> RawProgress for T
    where
        T: Progress,
    {
        fn init(&mut self, max: Option<Step>, unit: Option<Unit>) {
            <T as Progress>::init(self, max, unit)
        }

        fn set(&mut self, step: Step) {
            <T as Progress>::set(self, step)
        }

        fn unit(&self) -> Option<Unit> {
            <T as Progress>::unit(self)
        }

        fn max(&self) -> Option<Step> {
            <T as Progress>::max(self)
        }

        fn set_max(&mut self, max: Option<Step>) -> Option<Step> {
            <T as Progress>::set_max(self, max)
        }

        fn step(&self) -> Step {
            <T as Progress>::step(self)
        }

        fn inc_by(&mut self, step: Step) {
            <T as Progress>::inc_by(self, step)
        }

        fn inc(&mut self) {
            <T as Progress>::inc(self)
        }

        fn set_name(&mut self, name: String) {
            <T as Progress>::set_name(self, name)
        }

        fn name(&self) -> Option<String> {
            <T as Progress>::name(self)
        }

        fn id(&self) -> Id {
            <T as Progress>::id(self)
        }

        fn message(&self, level: MessageLevel, message: String) {
            <T as Progress>::message(self, level, message)
        }

        fn counter(&self) -> Option<StepShared> {
            <T as Progress>::counter(self)
        }

        fn info(&self, message: String) {
            <T as Progress>::info(self, message)
        }

        fn done(&self, message: String) {
            <T as Progress>::done(self, message)
        }

        fn fail(&self, message: String) {
            <T as Progress>::fail(self, message)
        }

        fn show_throughput(&self, start: Instant) {
            <T as Progress>::show_throughput(self, start)
        }

        fn show_throughput_with(&self, start: Instant, step: Step, unit: Unit, level: MessageLevel) {
            <T as Progress>::show_throughput_with(self, start, step, unit, level)
        }
    }

    impl<'a, T> Progress for &'a mut T
    where
        T: Progress,
    {
        type SubProgress = <T as Progress>::SubProgress;

        fn add_child(&mut self, name: impl Into<String>) -> Self::SubProgress {
            self.deref_mut().add_child(name)
        }

        fn add_child_with_id(&mut self, name: impl Into<String>, id: Id) -> Self::SubProgress {
            self.deref_mut().add_child_with_id(name, id)
        }

        fn init(&mut self, max: Option<Step>, unit: Option<Unit>) {
            self.deref_mut().init(max, unit)
        }

        fn set(&mut self, step: Step) {
            self.deref_mut().set(step)
        }

        fn unit(&self) -> Option<Unit> {
            self.deref().unit()
        }

        fn max(&self) -> Option<Step> {
            self.deref().max()
        }

        fn set_max(&mut self, max: Option<Step>) -> Option<Step> {
            self.deref_mut().set_max(max)
        }

        fn step(&self) -> Step {
            self.deref().step()
        }

        fn inc_by(&mut self, step: Step) {
            self.deref_mut().inc_by(step)
        }

        fn inc(&mut self) {
            self.deref_mut().inc()
        }

        fn set_name(&mut self, name: impl Into<String>) {
            self.deref_mut().set_name(name)
        }

        fn name(&self) -> Option<String> {
            self.deref().name()
        }

        fn id(&self) -> Id {
            self.deref().id()
        }

        fn message(&self, level: MessageLevel, message: impl Into<String>) {
            self.deref().message(level, message)
        }

        fn counter(&self) -> Option<StepShared> {
            self.deref().counter()
        }

        fn info(&self, message: impl Into<String>) {
            self.deref().info(message)
        }

        fn done(&self, message: impl Into<String>) {
            self.deref().done(message)
        }

        fn fail(&self, message: impl Into<String>) {
            self.deref().fail(message)
        }

        fn show_throughput(&self, start: Instant) {
            self.deref().show_throughput(start)
        }

        fn show_throughput_with(&self, start: Instant, step: Step, unit: Unit, level: MessageLevel) {
            self.deref().show_throughput_with(start, step, unit, level)
        }
    }
}
