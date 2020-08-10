use std::time::SystemTime;

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
#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct MessageRingBuffer {
    pub(crate) buf: Vec<Message>,
    cursor: usize,
    total: usize,
}

impl MessageRingBuffer {
    pub fn with_capacity(capacity: usize) -> MessageRingBuffer {
        MessageRingBuffer {
            buf: Vec::with_capacity(capacity),
            cursor: 0,
            total: 0,
        }
    }

    fn has_capacity(&self) -> bool {
        self.buf.len() < self.buf.capacity()
    }

    pub fn push_overwrite(&mut self, level: MessageLevel, origin: String, message: impl Into<String>) {
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
        self.total = self.total.wrapping_add(1);
    }

    pub fn copy_all(&self, out: &mut Vec<Message>) {
        out.clear();
        if self.buf.is_empty() {
            return;
        }
        out.extend_from_slice(&self.buf[self.cursor % self.buf.len()..]);
        if self.cursor != self.buf.len() {
            out.extend_from_slice(&self.buf[..self.cursor]);
        }
    }

    pub fn copy_new(&self, out: &mut Vec<Message>, prev: Option<MessageCopyState>) -> MessageCopyState {
        out.clear();
        match prev {
            Some(MessageCopyState { cursor, buf_len, total }) => {
                if self.total.saturating_sub(total) >= self.buf.capacity() {
                    self.copy_all(out);
                } else {
                    let new_elements_below_cap = self.buf.len().saturating_sub(buf_len);
                    let cursor_ofs: isize = self.cursor as isize - cursor as isize;
                    match cursor_ofs {
                        // there was some capacity left without wrapping around
                        c if c == 0 => {
                            out.extend_from_slice(&self.buf[self.buf.len() - new_elements_below_cap..]);
                        }
                        // cursor advanced
                        c if c > 0 => {
                            out.extend_from_slice(&self.buf[(cursor % self.buf.len())..self.cursor]);
                        }
                        // cursor wrapped around
                        c if c < 0 => {
                            out.extend_from_slice(&self.buf[(cursor % self.buf.len())..]);
                            out.extend_from_slice(&self.buf[..self.cursor]);
                        }
                        _ => unreachable!("logic dictates that… yeah, you really shouldn't ever see this!"),
                    }
                }
            }
            None => self.copy_all(out),
        };
        MessageCopyState {
            cursor: self.cursor,
            buf_len: self.buf.len(),
            total: self.total,
        }
    }
}

/// State used to keep track of what's new since the last time message were copied.
///
/// Note that due to the nature of a ring buffer, there is no guarantee that you see all messages.
pub struct MessageCopyState {
    cursor: usize,
    buf_len: usize,
    total: usize,
}
