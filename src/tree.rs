use crate::TreeOptions;
use dashmap::DashMap;
use parking_lot::Mutex;
use std::{sync::Arc, time::SystemTime};

/// The top-level of the progress tree.
#[derive(Clone, Debug)]
pub struct Root {
    pub(crate) inner: Arc<Mutex<Item>>,
}

impl Root {
    /// Create a new tree with default configuration.
    ///
    /// As opposed to [Item](./struct.Item.html) instances, this type can be closed and sent
    /// safely across threads.
    pub fn new() -> Root {
        TreeOptions::default().into()
    }

    /// Returns the maximum amount of messages we can keep before overwriting older ones.
    pub fn messages_capacity(&self) -> usize {
        self.inner.lock().messages.lock().buf.capacity()
    }

    /// Returns the current amount of `Item`s stored in the tree.
    /// **Note** that this is at most a guess as tasks can be added and removed in parallel.
    pub fn num_tasks(&self) -> usize {
        self.inner.lock().tree.len()
    }

    /// Adds a new child `tree::Item`, whose parent is this instance, with the given `name`.
    ///
    /// This builds a hierarchy of `tree::Item`s, each having their own progress.
    /// Use this method to [track progress](./struct.Item.html) of your first tasks.
    pub fn add_child(&self, name: impl Into<String>) -> Item {
        self.inner.lock().add_child(name)
    }

    /// Copy the entire progress tree into the given `out` vector, so that
    /// it can be traversed from beginning to end in order of hierarchy.
    pub fn sorted_snapshot(&self, out: &mut Vec<(Key, Value)>) {
        out.clear();
        out.extend(
            self.inner
                .lock()
                .tree
                .iter()
                .map(|r| (r.key().clone(), r.value().clone())),
        );
        out.sort_by_key(|t| t.0);
    }

    /// Copy all messages from the internal ring buffer into the given `out`
    /// vector. Messages are ordered from oldest to newest.
    pub fn copy_messages(&self, out: &mut Vec<Message>) {
        self.inner.lock().messages.lock().copy_into(out);
    }
}

/// The severity of a message
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MessageLevel {
    /// Rarely sent information related to the progress, not to be confused with the progress itself
    Info,
    /// Used to indicate that a task has failed, along with the reason
    Failure,
    /// Indicates a task was completed successfully
    Success,
}

/// A message to be stored along with the progress tree.
///
/// It is created by [`Tree::message(…)`](./struct.Item.html#method.message).
#[derive(Debug, Clone)]
pub struct Message {
    /// The time at which the message was sent.
    pub time: SystemTime,
    /// The serverity of the message
    pub level: MessageLevel,
    /// The name of the task that created the `Message`
    pub origin: String,
    /// The message itself
    pub message: String,
}

#[derive(Debug)]
pub(crate) struct MessageRingBuffer {
    buf: Vec<Message>,
    cursor: usize,
}

impl MessageRingBuffer {
    pub fn with_capacity(capacity: usize) -> MessageRingBuffer {
        MessageRingBuffer {
            buf: Vec::with_capacity(capacity),
            cursor: 0,
        }
    }

    fn has_capacity(&self) -> bool {
        self.buf.len() < self.buf.capacity()
    }

    pub fn push_overwrite(&mut self, level: MessageLevel, origin: String, message: &str) {
        let msg = Message {
            time: SystemTime::now(),
            level,
            origin,
            message: message.to_string(),
        };
        if self.has_capacity() {
            self.buf.push(msg)
        } else {
            self.buf[self.cursor] = msg;
            self.cursor = (self.cursor + 1) % self.buf.len();
        }
    }

    pub fn copy_into(&self, out: &mut Vec<Message>) {
        out.clear();
        if self.has_capacity() {
            out.extend_from_slice(self.buf.as_slice());
        } else {
            out.extend_from_slice(&self.buf[(self.cursor + 1) % self.buf.len()..]);
            if self.cursor + 1 != self.buf.len() {
                out.extend_from_slice(&self.buf[..self.cursor]);
            }
        }
    }
}

/// A `Tree` represents an element of the progress tree.
///
/// It can be used to set progress and send messages.
/// ```rust
/// let tree = prodash::Tree::new();
/// let mut progress = tree.add_child("task 1");
///
/// progress.init(Some(10), Some("elements"));
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
    pub(crate) highest_child_id: ItemId,
    pub(crate) tree: Arc<DashMap<Key, Value>>,
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
    pub fn init(&mut self, max: Option<ProgressStep>, unit: Option<&'static str>) {
        self.tree.get_mut(&self.key).map(|mut r| {
            r.value_mut().progress = Some(Progress {
                done_at: max,
                unit,
                ..Default::default()
            })
        });
    }

    fn alter_progress(&mut self, f: impl FnMut(&mut Progress)) {
        self.tree.get_mut(&self.key).map(|mut r| {
            // NOTE: since we wrap around, if there are more tasks than we can have IDs for,
            // and if all these tasks are still alive, two progress trees may see the same ID
            // when these go out of scope, they delete the key and the other tree will not find
            // its value anymore. Besides, it's probably weird to see tasks changing their progress
            // all the time…
            r.value_mut().progress.as_mut().map(f);
        });
    }

    /// Set the name of this task's progress to the given `name`.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.tree.get_mut(&self.key).map(|mut r| {
            r.value_mut().name = name.into();
        });
    }

    /// Set the current progress to the given `step`.
    ///
    /// **Note**: that this call has no effect unless `init(…)` was called before.
    pub fn set(&mut self, step: ProgressStep) {
        self.alter_progress(|p| {
            p.step = step;
            p.state = ProgressState::Running;
        });
    }

    /// Call to indicate that progress cannot be made.
    ///
    /// If `eta` is `Some(…)`, it specifies the time at which this task is expected to
    /// make progress again.
    ///
    /// The blocked-state is undone next time [`tree::Item::set(…)`](./struct.Item.html#method.set) is called.
    pub fn blocked(&mut self, eta: Option<SystemTime>) {
        self.alter_progress(|p| p.state = ProgressState::Blocked(eta));
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
            Value {
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
    pub fn message(&mut self, level: MessageLevel, message: impl AsRef<str>) {
        self.messages.lock().push_overwrite(
            level,
            {
                let name = self
                    .tree
                    .get(&self.key)
                    .map(|v| v.name.to_owned())
                    .unwrap_or_default();

                #[cfg(feature = "log-renderer")]
                match level {
                    MessageLevel::Failure => crate::warn!("{} → {}", name, message.as_ref()),
                    MessageLevel::Info | MessageLevel::Success => {
                        crate::info!("{} → {}", name, message.as_ref())
                    }
                };

                name
            },
            message.as_ref(),
        )
    }

    /// Create a message indicating the task is done
    pub fn done(&mut self, message: impl AsRef<str>) {
        self.message(MessageLevel::Success, message)
    }

    /// Create a message indicating the task failed
    pub fn fail(&mut self, message: impl AsRef<str>) {
        self.message(MessageLevel::Failure, message)
    }

    /// Create a message providing additional information about the progress thus far.
    pub fn info(&mut self, message: impl AsRef<str>) {
        self.message(MessageLevel::Info, message)
    }
}

type ItemId = u16; // NOTE: This means we will show weird behaviour if there are more than 2^16 tasks at the same time on a level
/// The amount of steps a progress can make
pub type ProgressStep = u32;

/// A type identifying a spot in the hierarchy of `Tree` items.
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Key(
    (
        Option<ItemId>,
        Option<ItemId>,
        Option<ItemId>,
        Option<ItemId>,
    ),
);

impl Key {
    fn add_child(self, child_id: ItemId) -> Key {
        Key(match self {
            Key((None, None, None, None)) => (Some(child_id), None, None, None),
            Key((a, None, None, None)) => (a, Some(child_id), None, None),
            Key((a, b, None, None)) => (a, b, Some(child_id), None),
            Key((a, b, c, None)) => (a, b, c, Some(child_id)),
            Key((a, b, c, _d)) => {
                crate::warn!("Maximum nesting level reached. Adding tasks to current parent");
                (a, b, c, Some(child_id))
            }
        })
    }

    pub fn level(&self) -> u8 {
        match self {
            Key((None, None, None, None)) => 0,
            Key((Some(_), None, None, None)) => 1,
            Key((Some(_), Some(_), None, None)) => 2,
            Key((Some(_), Some(_), Some(_), None)) => 3,
            Key((Some(_), Some(_), Some(_), Some(_))) => 4,
            _ => unreachable!("This is a bug - Keys follow a certain pattern"),
        }
    }

    pub const fn max_level() -> u8 {
        4
    }
}

/// Indicate whether a progress can or cannot be made.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum ProgressState {
    /// Indicates a task is blocked and cannot make progress, optionally until the
    /// given time.
    Blocked(Option<SystemTime>),
    /// The task is running
    Running,
}

impl Default for ProgressState {
    fn default() -> Self {
        ProgressState::Running
    }
}

/// Progress associated with some item in the progress tree.
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Progress {
    /// The amount of progress currently made
    pub step: ProgressStep,
    /// The step at which no further progress has to be made.
    ///
    /// If unset, the progress is unbounded.
    pub done_at: Option<ProgressStep>,
    /// The unit associated with the progress.
    pub unit: Option<&'static str>,
    /// Whether progress can be made or not
    pub state: ProgressState,
}

impl Progress {
    /// Returns a number between `Some(0.0)` and `Some(1.0)`, or `None` if the progress is unbounded.
    ///
    /// A task half done would return `Some(0.5)`.
    pub fn fraction(&self) -> Option<f32> {
        self.done_at
            .map(|done_at| self.step as f32 / done_at as f32)
    }
}

/// The value associated with a spot in the hierarchy.
#[derive(Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Value {
    /// The name of the `Item` or task.
    pub name: String,
    /// The progress itself, unless this value belongs to an `Item` serving as organizational unit.
    pub progress: Option<Progress>,
}
