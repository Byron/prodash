use dashmap::DashMap;

use crate::messages::MessageRingBuffer;

/// The top-level of the progress tree.
#[derive(Debug)]
pub struct Root {
    pub(crate) inner: parking_lot::Mutex<Item>,
}

/// A `Tree` represents an element of the progress tree.
///
/// It can be used to set progress and send messages.
/// ```rust
/// let tree = prodash::tree::Root::new();
/// let mut progress = tree.add_child("task 1");
///
/// progress.init(Some(10), Some("elements".into()));
/// for p in 0..10 {
///     progress.set(p);
/// }
/// progress.done("great success");
/// let mut  sub_progress = progress.add_child_with_id("sub-task 1", *b"TSK2");
/// sub_progress.init(None, None);
/// sub_progress.set(5);
/// sub_progress.fail("couldn't finish");
/// ```
#[derive(Debug)]
pub struct Item {
    pub(crate) key: crate::progress::Key,
    pub(crate) value: crate::progress::StepShared,
    pub(crate) highest_child_id: crate::progress::key::Id,
    pub(crate) tree: std::sync::Arc<DashMap<crate::progress::Key, crate::progress::Task>>,
    pub(crate) messages: std::sync::Arc<parking_lot::Mutex<MessageRingBuffer>>,
}

mod item;
///
pub mod root;

#[cfg(test)]
mod tests;
