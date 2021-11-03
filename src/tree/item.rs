use crate::{
    messages::{Envelope, Message, MessageLevel, MessageRingBuffer},
    progress::{key, Key, State, Step, Task, Value},
    unit::Unit,
};
use dashmap::DashMap;
use parking_lot::Mutex;
use std::{ops::Deref, sync::Arc, time::SystemTime};

/// A `Tree` represents an element of the progress tree.
///
/// It can be used to set progress and send messages.
/// ```rust
/// let tree = prodash::Tree::new();
/// let mut progress = tree.add_child("task 1");
///
/// progress.init(Some(10), Some("elements".into()));
/// for p in 0..10 {
///     progress.set(p);
/// }
/// progress.done("great success");
/// let mut  sub_progress = progress.add_child("sub-task 1");
/// sub_progress.init(None, None);
/// sub_progress.set(5);
/// sub_progress.fail("couldn't finish");
/// ```
#[derive(Debug)]
pub struct Item {
    pub(crate) key: Key,
    pub(crate) highest_child_id: key::Id,
    pub(crate) tree: Arc<DashMap<Key, Task>>,
    pub(crate) messages: Arc<Mutex<MessageRingBuffer>>,
}

impl Drop for Item {
    fn drop(&mut self) {
        self.tree.remove(&self.key);
    }
}

impl Item {
    /// Initialize the Item for receiving progress information.
    ///
    /// If `max` is `Some(…)`, it will be treated as upper bound. When progress is [set(…)](./struct.Item.html#method.set)
    /// it should not exceed the given maximum.
    /// If `max` is `None`, the progress is unbounded. Use this if the amount of work cannot accurately
    /// be determined.
    ///
    /// If `unit` is `Some(…)`, it is used for display purposes only. It should be using the plural.
    ///
    /// If this method is never called, this `Item` will serve as organizational unit, useful to add more structure
    /// to the progress tree.
    ///
    /// **Note** that this method can be called multiple times, changing the bounded-ness and unit at will.
    pub fn init(&mut self, max: Option<Step>, unit: Option<Unit>) {
        if let Some(mut r) = self.tree.get_mut(&self.key) {
            r.value_mut().progress = Some(Value {
                done_at: max,
                unit,
                ..Default::default()
            })
        };
    }

    fn alter_progress(&mut self, f: impl FnMut(&mut Value)) {
        if let Some(mut r) = self.tree.get_mut(&self.key) {
            // NOTE: since we wrap around, if there are more tasks than we can have IDs for,
            // and if all these tasks are still alive, two progress trees may see the same ID
            // when these go out of scope, they delete the key and the other tree will not find
            // its value anymore. Besides, it's probably weird to see tasks changing their progress
            // all the time…
            r.value_mut().progress.as_mut().map(f);
        };
    }

    /// Set the name of this task's progress to the given `name`.
    pub fn set_name(&mut self, name: impl Into<String>) {
        if let Some(mut r) = self.tree.get_mut(&self.key) {
            r.value_mut().name = name.into();
        };
    }

    /// Get the name of this task's progress
    pub fn name(&self) -> Option<String> {
        self.tree.get(&self.key).map(|r| r.value().name.to_owned())
    }

    /// Returns the current step, as controlled by `inc*(…)` calls
    pub fn step(&self) -> Option<Step> {
        self.tree
            .get(&self.key)
            .and_then(|r| r.value().progress.as_ref().map(|p| p.step))
    }

    /// Returns the maximum about of items we expect, as provided with the `init(…)` call
    pub fn max(&self) -> Option<Step> {
        self.tree
            .get(&self.key)
            .and_then(|r| r.value().progress.as_ref().and_then(|p| p.done_at))
    }

    /// Returns the (cloned) unit associated with this Progress
    pub fn unit(&self) -> Option<Unit> {
        self.tree
            .get(&self.key)
            .and_then(|r| r.value().progress.as_ref().and_then(|p| p.unit.clone()))
    }

    /// Set the current progress to the given `step`.
    ///
    /// **Note**: that this call has no effect unless `init(…)` was called before.
    pub fn set(&mut self, step: Step) {
        self.alter_progress(|p| {
            p.step = step;
            p.state = State::Running;
        });
    }

    /// Increment the current progress by the given `step`.
    ///
    /// **Note**: that this call has no effect unless `init(…)` was called before.
    pub fn inc_by(&mut self, step: Step) {
        self.alter_progress(|p| {
            p.step += step;
            p.state = State::Running;
        });
    }

    /// Increment the current progress by one.
    ///
    /// **Note**: that this call has no effect unless `init(…)` was called before.
    pub fn inc(&mut self) {
        self.alter_progress(|p| {
            p.step += 1;
            p.state = State::Running;
        });
    }

    /// Call to indicate that progress cannot be indicated, and that the task cannot be interrupted.
    /// Use this, as opposed to `halted(…)`, if a non-interruptable call is about to be made without support
    /// for any progress indication.
    ///
    /// If `eta` is `Some(…)`, it specifies the time at which this task is expected to
    /// make progress again.
    ///
    /// The blocked-state is undone next time [`tree::Item::set(…)`](./struct.Item.html#method.set) is called.
    pub fn blocked(&mut self, reason: &'static str, eta: Option<SystemTime>) {
        self.alter_progress(|p| p.state = State::Blocked(reason, eta));
    }

    /// Call to indicate that progress cannot be indicated, even though the task can be interrupted.
    /// Use this, as opposed to `blocked(…)`, if an interruptable call is about to be made without support
    /// for any progress indication.
    ///
    /// If `eta` is `Some(…)`, it specifies the time at which this task is expected to
    /// make progress again.
    ///
    /// The halted-state is undone next time [`tree::Item::set(…)`](./struct.Item.html#method.set) is called.
    pub fn halted(&mut self, reason: &'static str, eta: Option<SystemTime>) {
        self.alter_progress(|p| p.state = State::Halted(reason, eta));
    }

    /// Adds a new child `Tree`, whose parent is this instance, with the given `name`.
    ///
    /// **Important**: The depth of the hierarchy is limited to [`tree::Key::max_level`](./struct.Key.html#method.max_level).
    /// Exceeding the level will be ignored, and new tasks will be added to this instance's
    /// level instead.
    pub fn add_child(&mut self, name: impl Into<String>) -> Item {
        let child_key = self.key.add_child(self.highest_child_id);
        self.tree.insert(
            child_key,
            Task {
                name: name.into(),
                progress: None,
            },
        );
        self.highest_child_id = self.highest_child_id.wrapping_add(1);
        Item {
            highest_child_id: 0,
            key: child_key,
            tree: self.tree.clone(),
            messages: self.messages.clone(),
        }
    }

    /// Create a `message` of the given `level` and store it with the progress tree.
    ///
    /// Use this to provide additional,human-readable information about the progress
    /// made, including indicating success or failure.
    pub fn message(&mut self, level: MessageLevel, message: impl Into<String>) {
        let message: String = message.into();
        self.messages.lock().push_overwrite(Envelope::Message(Message::new(
            level,
            {
                let name = self.tree.get(&self.key).map(|v| v.name.to_owned()).unwrap_or_default();

                #[cfg(feature = "progress-tree-log")]
                match level {
                    MessageLevel::Failure => crate::warn!("{} → {}", name, message),
                    MessageLevel::Info | MessageLevel::Success => crate::info!("{} → {}", name, message),
                };

                name
            },
            message,
        )))
    }

    /// Create a message indicating the task is done
    pub fn done(&mut self, message: impl Into<String>) {
        self.message(MessageLevel::Success, message)
    }

    /// Create a message indicating the task failed
    pub fn fail(&mut self, message: impl Into<String>) {
        self.message(MessageLevel::Failure, message)
    }

    /// Create a message providing additional information about the progress thus far.
    pub fn info(&mut self, message: impl Into<String>) {
        self.message(MessageLevel::Info, message)
    }

    pub(crate) fn deep_clone(&self) -> Item {
        Item {
            key: self.key,
            highest_child_id: self.highest_child_id,
            tree: Arc::new(self.tree.deref().clone()),
            messages: Arc::new(Mutex::new(self.messages.lock().clone())),
        }
    }
}

impl crate::Progress for Item {
    type SubProgress = Item;

    fn add_child(&mut self, name: impl Into<String>) -> Self::SubProgress {
        Item::add_child(self, name)
    }

    fn init(&mut self, max: Option<usize>, unit: Option<Unit>) {
        Item::init(self, max, unit)
    }

    fn set(&mut self, step: usize) {
        Item::set(self, step)
    }

    fn unit(&self) -> Option<Unit> {
        Item::unit(self)
    }

    fn max(&self) -> Option<usize> {
        Item::max(self)
    }

    fn step(&self) -> usize {
        Item::step(self).unwrap_or(0)
    }

    fn inc_by(&mut self, step: usize) {
        self.inc_by(step)
    }

    fn set_name(&mut self, name: impl Into<String>) {
        Item::set_name(self, name)
    }

    fn name(&self) -> Option<String> {
        Item::name(self)
    }

    fn message(&mut self, level: MessageLevel, message: impl Into<String>) {
        Item::message(self, level, message)
    }
}
