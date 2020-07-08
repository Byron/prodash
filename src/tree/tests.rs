mod message_buffer {
    use crate::tree::{Message, MessageLevel, MessageRingBuffer};

    fn push(buf: &mut MessageRingBuffer, msg: impl Into<String>) {
        buf.push_overwrite(MessageLevel::Info, "test".into(), msg);
    }

    fn messages(msg: &[&'static str]) -> Vec<Message> {
        msg.iter()
            .map(|msg| Message {
                time: std::time::SystemTime::now(),
                level: MessageLevel::Info,
                origin: "test".into(),
                message: msg.to_string(),
            })
            .collect()
    }
    fn assert_messages(actual: &Vec<Message>, expected: &[&'static str]) {
        let actual: Vec<_> = actual.iter().map(|m| m.message.as_str()).collect();
        assert_eq!(
            expected,
            actual.as_slice(),
            "messages are ordered old to new"
        );
    }

    #[test]
    fn copy_all() {
        let mut buf = MessageRingBuffer::with_capacity(2);
        let mut out = Vec::new();
        push(&mut buf, "one");
        buf.copy_all(&mut out);
        assert_eq!(out, buf.buf);

        push(&mut buf, "two");
        buf.copy_all(&mut out);
        assert_eq!(out, buf.buf);

        push(&mut buf, "three");
        buf.copy_all(&mut out);
        assert_messages(&out, &["two", "three"]);

        push(&mut buf, "four");
        buf.copy_all(&mut out);
        assert_messages(&out, &["three", "four"]);

        push(&mut buf, "five");
        buf.copy_all(&mut out);
        assert_messages(&out, &["four", "five"]);
    }
}
mod key_adjacency {
    use crate::tree::SiblingLocation::*;
    use crate::tree::{Adjacency, Key, Value};

    fn to_kv(keys: &[Key]) -> Vec<(Key, Value)> {
        let mut v: Vec<_> = keys
            .iter()
            .map(|k| (k.to_owned(), Value::default()))
            .collect();
        v.sort_by_key(|v| v.0);
        v
    }

    fn root_with_two_children() -> Vec<(Key, Value)> {
        let r = Key::default();
        to_kv(&[r.add_child(1), r.add_child(2)][..])
    }
    fn root_with_two_children_with_two_children() -> Vec<(Key, Value)> {
        let r = Key::default();
        let p1 = r.add_child(1);
        let p2 = r.add_child(2);
        to_kv(
            &[
                p1.clone(),
                p1.add_child(1),
                p1.add_child(2),
                p2.clone(),
                p2.add_child(1),
                p2.add_child(2),
            ][..],
        )
    }

    fn root_with_three_levels() -> Vec<(Key, Value)> {
        let r = Key::default();
        let p1 = r.add_child(1);
        let p2 = p1.add_child(2);
        to_kv(&[p1, p2.clone(), p2.add_child(1)][..])
    }

    fn root_with_three_levels_two_siblings_on_level_2() -> Vec<(Key, Value)> {
        let r = Key::default();
        let p1 = r.add_child(1);
        let p11 = p1.add_child(1);
        let p12 = p1.add_child(2);
        to_kv(
            &[
                p1,
                p11.clone(),
                p11.add_child(1),
                p12.clone(),
                p12.add_child(1),
            ][..],
        )
    }

    #[test]
    fn root_level() {
        let entries = root_with_two_children();
        assert_eq!(
            Key::adjacency(&entries, 0),
            Adjacency(AboveAndBelow, NotFound, NotFound, NotFound)
        );
        assert_eq!(
            Key::adjacency(&entries, 1),
            Adjacency(Above, NotFound, NotFound, NotFound)
        );
    }

    #[test]
    fn level_2_two_siblings() {
        let entries = root_with_two_children_with_two_children();
        assert_eq!(
            Key::adjacency(&entries, 0),
            Adjacency(AboveAndBelow, NotFound, NotFound, NotFound)
        );
        {
            assert_eq!(
                Key::adjacency(&entries, 1),
                Adjacency(AboveAndBelow, AboveAndBelow, NotFound, NotFound)
            );
            assert_eq!(
                Key::adjacency(&entries, 2),
                Adjacency(AboveAndBelow, Above, NotFound, NotFound)
            );
        }
        assert_eq!(
            Key::adjacency(&entries, 3),
            Adjacency(Above, NotFound, NotFound, NotFound)
        );
        {
            assert_eq!(
                Key::adjacency(&entries, 4),
                Adjacency(NotFound, AboveAndBelow, NotFound, NotFound)
            );
            assert_eq!(
                Key::adjacency(&entries, 5),
                Adjacency(NotFound, Above, NotFound, NotFound)
            );
        }
    }

    #[test]
    fn level_3_single_sibling() {
        let entries = root_with_three_levels();
        assert_eq!(
            Key::adjacency(&entries, 0),
            Adjacency(Above, NotFound, NotFound, NotFound)
        );
        {
            assert_eq!(
                Key::adjacency(&entries, 1),
                Adjacency(NotFound, Above, NotFound, NotFound)
            );
            {
                assert_eq!(
                    Key::adjacency(&entries, 2),
                    Adjacency(NotFound, NotFound, Above, NotFound)
                );
            }
        }
    }

    #[test]
    fn level_3_two_siblings() {
        let entries = root_with_three_levels_two_siblings_on_level_2();
        {
            assert_eq!(
                Key::adjacency(&entries, 0),
                Adjacency(Above, NotFound, NotFound, NotFound)
            );
            {
                assert_eq!(
                    Key::adjacency(&entries, 1),
                    Adjacency(NotFound, AboveAndBelow, NotFound, NotFound)
                );
                {
                    assert_eq!(
                        Key::adjacency(&entries, 2),
                        Adjacency(NotFound, AboveAndBelow, Above, NotFound)
                    );
                }

                assert_eq!(
                    Key::adjacency(&entries, 3),
                    Adjacency(NotFound, Above, NotFound, NotFound)
                );
                {
                    assert_eq!(
                        Key::adjacency(&entries, 4),
                        Adjacency(NotFound, NotFound, Above, NotFound)
                    );
                }
            }
        }
    }

    #[test]
    fn orphaned_child_node() {
        let mut entries = root_with_two_children();
        entries.insert(
            1,
            (
                Key::default().add_child(0).add_child(0).add_child(1),
                Value::default(),
            ),
        );
        entries.sort_by_key(|v| v.0);
        assert_eq!(
            Key::adjacency(&entries, 0),
            Adjacency(AboveAndBelow, NotFound, NotFound, NotFound),
        );
        assert_eq!(
            Key::adjacency(&entries, 1),
            Adjacency(AboveAndBelow, NotFound, NotFound, NotFound)
        );
        assert_eq!(
            Key::adjacency(&entries, 2),
            Adjacency(Above, NotFound, NotFound, NotFound)
        );
    }
}
