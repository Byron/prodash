use crate::TreeOptions;
use dashmap::DashMap;
use parking_lot::Mutex;
use std::ops::{Index, IndexMut};
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
    /// The severity of the message
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

    pub fn push_overwrite(
        &mut self,
        level: MessageLevel,
        origin: String,
        message: impl Into<String>,
    ) {
        let msg = Message {
            time: SystemTime::now(),
            level,
            origin,
            message: message.into(),
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

    /// Get the name of this task's progress
    pub fn name(&self) -> Option<String> {
        self.tree.get(&self.key).map(|r| r.value().name.to_owned())
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

    /// Call to indicate that progress cannot be indicated, and that the task cannot be interrupted.
    /// Use this, as opposed to `halted(…)`, if a non-interruptable call is about to be made without support
    /// for any progress indication.
    ///
    /// If `eta` is `Some(…)`, it specifies the time at which this task is expected to
    /// make progress again.
    ///
    /// The blocked-state is undone next time [`tree::Item::set(…)`](./struct.Item.html#method.set) is called.
    pub fn blocked(&mut self, reason: &'static str, eta: Option<SystemTime>) {
        self.alter_progress(|p| p.state = ProgressState::Blocked(reason, eta));
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
        self.alter_progress(|p| p.state = ProgressState::Halted(reason, eta));
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
    pub fn message(&mut self, level: MessageLevel, message: impl Into<String> + std::fmt::Display) {
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
                    MessageLevel::Failure => crate::warn!("{} → {}", name, message),
                    MessageLevel::Info | MessageLevel::Success => {
                        crate::info!("{} → {}", name, message)
                    }
                };

                name
            },
            message,
        )
    }

    /// Create a message indicating the task is done
    pub fn done(&mut self, message: impl Into<String> + std::fmt::Display) {
        self.message(MessageLevel::Success, message)
    }

    /// Create a message indicating the task failed
    pub fn fail(&mut self, message: impl Into<String> + std::fmt::Display) {
        self.message(MessageLevel::Failure, message)
    }

    /// Create a message providing additional information about the progress thus far.
    pub fn info(&mut self, message: impl Into<String> + std::fmt::Display) {
        self.message(MessageLevel::Info, message)
    }
}

type ItemId = u16; // NOTE: This means we will show weird behaviour if there are more than 2^16 tasks at the same time on a level
pub type Level = u8; // a level in the hierarchy of key components

/// The amount of steps a progress can make
pub type ProgressStep = u32;

/// A type identifying a spot in the hierarchy of `Tree` items.
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Key(
    Option<ItemId>,
    Option<ItemId>,
    Option<ItemId>,
    Option<ItemId>,
);

/// Determines if a sibling is above or below in the given level of hierarchy
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub(crate) enum SiblingLocation {
    Above,
    Below,
    AboveAndBelow,
    NotFound,
}

impl SiblingLocation {
    fn merge(&mut self, other: SiblingLocation) {
        use SiblingLocation::*;
        *self = match (*self, other) {
            (any, NotFound) => any,
            (NotFound, any) => any,
            (Above, Below) => AboveAndBelow,
            (Below, Above) => AboveAndBelow,
            (AboveAndBelow, _) => AboveAndBelow,
            (_, AboveAndBelow) => AboveAndBelow,
            (Above, Above) => Above,
            (Below, Below) => Below,
        };
    }
}

impl Default for SiblingLocation {
    fn default() -> Self {
        SiblingLocation::NotFound
    }
}

/// A type providing information about what's above and below `Tree` items.
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub(crate) struct Adjacency(
    pub SiblingLocation,
    pub SiblingLocation,
    pub SiblingLocation,
    pub SiblingLocation,
);

impl Adjacency {
    pub(crate) fn level(&self) -> Level {
        use SiblingLocation::*;
        match self {
            Adjacency(NotFound, NotFound, NotFound, NotFound) => 0,
            Adjacency(_a, NotFound, NotFound, NotFound) => 1,
            Adjacency(_a, _b, NotFound, NotFound) => 2,
            Adjacency(_a, _b, _c, NotFound) => 3,
            Adjacency(_a, _b, _c, _d) => 4,
        }
    }
    pub fn get(&self, level: Level) -> Option<&SiblingLocation> {
        Some(match level {
            1 => &self.0,
            2 => &self.1,
            3 => &self.2,
            4 => &self.3,
            _ => return None,
        })
    }
    pub fn get_mut(&mut self, level: Level) -> Option<&mut SiblingLocation> {
        Some(match level {
            1 => &mut self.0,
            2 => &mut self.1,
            3 => &mut self.2,
            4 => &mut self.3,
            _ => return None,
        })
    }
}

impl Index<Level> for Adjacency {
    type Output = SiblingLocation;
    fn index(&self, index: Level) -> &Self::Output {
        self.get(index).expect("adjacency index in bound")
    }
}
impl IndexMut<Level> for Adjacency {
    fn index_mut(&mut self, index: Level) -> &mut Self::Output {
        self.get_mut(index).expect("adjacency index in bound")
    }
}

impl Key {
    pub(crate) fn add_child(self, child_id: ItemId) -> Key {
        match self {
            Key(None, None, None, None) => Key(Some(child_id), None, None, None),
            Key(a, None, None, None) => Key(a, Some(child_id), None, None),
            Key(a, b, None, None) => Key(a, b, Some(child_id), None),
            Key(a, b, c, None) => Key(a, b, c, Some(child_id)),
            Key(a, b, c, _d) => {
                crate::warn!("Maximum nesting level reached. Adding tasks to current parent");
                Key(a, b, c, Some(child_id))
            }
        }
    }

    /// The level of hierarchy a node is placed in, i.e. the amount of path components
    pub fn level(&self) -> Level {
        match self {
            Key(None, None, None, None) => 0,
            Key(Some(_), None, None, None) => 1,
            Key(Some(_), Some(_), None, None) => 2,
            Key(Some(_), Some(_), Some(_), None) => 3,
            Key(Some(_), Some(_), Some(_), Some(_)) => 4,
            _ => unreachable!("This is a bug - Keys follow a certain pattern"),
        }
    }

    fn get(&self, level: Level) -> Option<&ItemId> {
        match level {
            1 => self.0.as_ref(),
            2 => self.1.as_ref(),
            3 => self.2.as_ref(),
            4 => self.3.as_ref(),
            _ => return None,
        }
    }

    pub(crate) fn shares_parent_with(&self, other: &Key, parent_level: Level) -> bool {
        if parent_level < 1 {
            return true;
        }
        for level in 1..=parent_level {
            if let (Some(lhs), Some(rhs)) = (self.get(level), other.get(level)) {
                if lhs != rhs {
                    return false;
                }
            } else {
                return false;
            }
        }
        return true;
    }

    /// Compute the adjacency map for the key in `sorted` at the given `index`.
    ///
    /// It's vital that the invariant of `sorted` to actually be sorted by key is upheld
    /// for the result to be reliable.
    pub(crate) fn adjacency(sorted: &[(Key, Value)], index: usize) -> Adjacency {
        use SiblingLocation::*;
        let key = &sorted[index].0;
        let key_level = key.level();
        let mut adjecency = Adjacency::default();
        if key_level == 0 {
            return adjecency;
        }

        fn search<'a>(
            iter: impl Iterator<Item = &'a (Key, Value)>,
            key: &Key,
            key_level: Level,
            current_level: Level,
            _id_at_level: ItemId,
        ) -> Option<usize> {
            iter.map(|(k, _)| k)
                .take_while(|other| key.shares_parent_with(other, current_level.saturating_sub(1)))
                .enumerate()
                .find(|(_idx, k)| {
                    if current_level == key_level {
                        k.level() == key_level || k.level() + 1 == key_level
                    } else {
                        k.level() == current_level
                    }
                })
                .map(|(idx, _)| idx)
        };

        let upward_iter = |from: usize, key: &Key, level: Level, id_at_level: ItemId| {
            search(
                sorted[..from].iter().rev(),
                key,
                key_level,
                level,
                id_at_level,
            )
        };
        let downward_iter = |from: usize, key: &Key, level: Level, id_at_level: ItemId| {
            sorted
                .get(from + 1..)
                .and_then(|s| search(s.iter(), key, key_level, level, id_at_level))
        };

        {
            let mut cursor = index;
            for level in (1..=key_level).rev() {
                if level == 1 {
                    adjecency[level].merge(Above); // the root or any other sibling on level one
                    continue;
                }
                if let Some(key_offset) = upward_iter(cursor, &key, level, key[level]) {
                    cursor = index.saturating_sub(key_offset);
                    adjecency[level].merge(Above);
                }
            }
        }
        {
            let mut cursor = index;
            for level in (1..=key_level).rev() {
                if let Some(key_offset) = downward_iter(cursor, &key, level, key[level]) {
                    cursor = index + key_offset;
                    adjecency[level].merge(Below);
                }
            }
        }
        for level in 1..key_level {
            if key_level == 1 && index + 1 == sorted.len() {
                continue;
            }
            adjecency[level] = match adjecency[level] {
                Above | Below | NotFound => NotFound,
                AboveAndBelow => AboveAndBelow,
            };
        }
        adjecency
    }

    /// The maximum amount of path components we can represent.
    pub const fn max_level() -> Level {
        4
    }
}

impl Index<Level> for Key {
    type Output = ItemId;

    fn index(&self, index: Level) -> &Self::Output {
        self.get(index).expect("key index in bound")
    }
}

/// Indicate whether a progress can or cannot be made.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum ProgressState {
    /// Indicates a task is blocked and cannot indicate progress, optionally until the
    /// given time. The task cannot easily be interrupted.
    Blocked(&'static str, Option<SystemTime>),
    /// Indicates a task cannot indicate progress, optionally until the
    /// given time. The task can be interrupted.
    Halted(&'static str, Option<SystemTime>),
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
