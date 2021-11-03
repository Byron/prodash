use crate::{
    messages::{Envelope, MessageCopyState, MessageRingBuffer},
    progress::{Key, Task},
    tree::Item,
};
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
    pub fn sorted_snapshot(&self, out: &mut Vec<(Key, Task)>) {
        out.clear();
        out.extend(self.inner.lock().tree.iter().map(|r| (*r.key(), r.value().clone())));
        out.sort_by_key(|t| t.0);
    }

    /// Create a raw `message` and store it with the progress tree.
    ///
    /// Use this to render additional unclassified output about the progress made.
    fn message_raw(&mut self, message: impl Into<String>) {
        let inner = self.inner.lock();
        let mut messages = inner.messages.lock();
        for line in message.into().lines() {
            messages.push_overwrite(Envelope::RawMessage(line.trim_end().to_owned()));
        }
    }

    /// Copy all messages from the internal ring buffer into the given `out`
    /// vector. Messages are ordered from oldest to newest.
    pub fn copy_messages(&self, out: &mut Vec<Envelope>) {
        self.inner.lock().messages.lock().copy_all(out);
    }

    /// Copy only new messages from the internal ring buffer into the given `out`
    /// vector. Messages are ordered from oldest to newest.
    pub fn copy_new_messages(&self, out: &mut Vec<Envelope>, prev: Option<MessageCopyState>) -> MessageCopyState {
        self.inner.lock().messages.lock().copy_new(out, prev)
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

impl crate::Root for Root {
    fn messages_capacity(&self) -> usize {
        self.messages_capacity()
    }

    fn num_tasks(&self) -> usize {
        self.num_tasks()
    }

    fn sorted_snapshot(&self, out: &mut Vec<(Key, Task)>) {
        self.sorted_snapshot(out)
    }

    fn message_raw(&mut self, message: impl Into<String>) {
        self.message_raw(message)
    }

    fn copy_messages(&self, out: &mut Vec<Envelope>) {
        self.copy_messages(out)
    }

    fn copy_new_messages(&self, out: &mut Vec<Envelope>, prev: Option<MessageCopyState>) -> MessageCopyState {
        self.copy_new_messages(out, prev)
    }
}
