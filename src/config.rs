use crate::{
    tree::{Item, Key, MessageRingBuffer},
    Tree,
};
use dashmap::DashMap;
use parking_lot::Mutex;
use std::sync::Arc;

/// A way to configure new [`tree::Root`](./tree/struct.Root.html) instances
/// ```rust
/// use prodash::{Tree, TreeOptions};
/// let tree = TreeOptions::default().create();
/// let tree2 = TreeOptions { message_buffer_capacity: 100, ..TreeOptions::default() }.create();
/// ```
#[derive(Clone, Debug)]
pub struct TreeOptions {
    /// The amount of items the tree can hold without being forced to allocate
    pub initial_capacity: usize,
    /// The amount of messages we can hold before we start overwriting old ones
    pub message_buffer_capacity: usize,
}

impl TreeOptions {
    /// Create a new [`Root`](./tree/struct.Root.html) instance from the
    /// configuration within.
    pub fn create(self) -> Tree {
        self.into()
    }
}

impl Default for TreeOptions {
    fn default() -> Self {
        TreeOptions {
            initial_capacity: 100,
            message_buffer_capacity: 20,
        }
    }
}

impl From<TreeOptions> for Tree {
    fn from(
        TreeOptions {
            initial_capacity,
            message_buffer_capacity,
        }: TreeOptions,
    ) -> Self {
        Tree {
            inner: Arc::new(Mutex::new(Item {
                highest_child_id: 0,
                key: Key::default(),
                tree: Arc::new(DashMap::with_capacity(initial_capacity)),
                messages: Arc::new(Mutex::new(MessageRingBuffer::with_capacity(
                    message_buffer_capacity,
                ))),
            })),
        }
    }
}
