use crate::tree::{messages::MessageRingBuffer, Item, Key, Message, MessageCopyState, Value};
use dashmap::DashMap;
use parking_lot::Mutex;
use std::sync::Arc;

/// The top-level of the progress tree.
#[derive(Clone, Debug)]
pub struct Root {
    pub(crate) inner: Arc<Mutex<Item>>,
}

impl Default for Root {
    fn default() -> Self {
        Self::new()
    }
}

impl Root {
    /// Create a new tree with default configuration.
    ///
    /// As opposed to [Item](./struct.Item.html) instances, this type can be closed and sent
    /// safely across threads.
    pub fn new() -> Root {
        Options::default().into()
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
        out.extend(self.inner.lock().tree.iter().map(|r| (*r.key(), r.value().clone())));
        out.sort_by_key(|t| t.0);
    }

    /// Copy all messages from the internal ring buffer into the given `out`
    /// vector. Messages are ordered from oldest to newest.
    pub fn copy_messages(&self, out: &mut Vec<Message>) {
        self.inner.lock().messages.lock().copy_all(out);
    }

    /// Copy only new messages from the internal ring buffer into the given `out`
    /// vector. Messages are ordered from oldest to newest.
    pub fn copy_new_messages(&self, out: &mut Vec<Message>, prev: Option<MessageCopyState>) -> MessageCopyState {
        self.inner.lock().messages.lock().copy_new(out, prev)
    }

    /// Return the amount of messages currently stored as well as the maximum amount we can ever store (i.e. capacity)
    pub fn message_buffer_usage(&self) -> (usize, usize) {
        let inner = self.inner.lock();
        let guard = inner.messages.lock();
        (guard.buf.len(), guard.buf.capacity())
    }

    /// Duplicate all content and return it.
    ///
    /// This is an expensive operation, whereas `clone()` is not as it is shallow.
    pub fn deep_clone(&self) -> Root {
        Root {
            inner: Arc::new(Mutex::new(self.inner.lock().deep_clone())),
        }
    }
}

/// A way to configure new [`tree::Root`](./tree/struct.Root.html) instances
/// ```rust
/// use prodash::{Tree, TreeOptions};
/// let tree = TreeOptions::default().create();
/// let tree2 = TreeOptions { message_buffer_capacity: 100, ..TreeOptions::default() }.create();
/// ```
#[derive(Clone, Debug)]
pub struct Options {
    /// The amount of items the tree can hold without being forced to allocate
    pub initial_capacity: usize,
    /// The amount of messages we can hold before we start overwriting old ones
    pub message_buffer_capacity: usize,
}

impl Options {
    /// Create a new [`Root`](./tree/struct.Root.html) instance from the
    /// configuration within.
    pub fn create(self) -> Root {
        self.into()
    }
}

impl Default for Options {
    fn default() -> Self {
        Options {
            initial_capacity: 100,
            message_buffer_capacity: 20,
        }
    }
}

impl From<Options> for Root {
    fn from(
        Options {
            initial_capacity,
            message_buffer_capacity,
        }: Options,
    ) -> Self {
        Root {
            inner: Arc::new(Mutex::new(Item {
                highest_child_id: 0,
                key: Key::default(),
                tree: Arc::new(DashMap::with_capacity(initial_capacity)),
                messages: Arc::new(Mutex::new(MessageRingBuffer::with_capacity(message_buffer_capacity))),
            })),
        }
    }
}
