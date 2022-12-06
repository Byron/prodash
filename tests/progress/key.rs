use prodash::progress::Key;

#[test]
fn size_in_memory() {
    assert_eq!(std::mem::size_of::<Key>(), 24);
}

mod adjacency {
    use prodash::progress::{
        key::{Adjacency, SiblingLocation::*},
        Key, Task,
    };

    fn to_kv(keys: &[Key]) -> Vec<(Key, Task)> {
        let mut v: Vec<_> = keys.iter().map(|k| (k.to_owned(), Task::default())).collect();
        v.sort_by_key(|v| v.0);
        v
    }

    fn root_with_two_children() -> Vec<(Key, Task)> {
        let r = Key::default();
        to_kv(&[r.add_child(1), r.add_child(2)][..])
    }
    fn root_with_two_children_with_two_children() -> Vec<(Key, Task)> {
        let r = Key::default();
        let p1 = r.add_child(1);
        let p2 = r.add_child(2);
        to_kv(
            &[
                p1,
                p1.add_child(1),
                p1.add_child(2),
                p2,
                p2.add_child(1),
                p2.add_child(2),
            ][..],
        )
    }

    fn root_with_three_levels() -> Vec<(Key, Task)> {
        let r = Key::default();
        let p1 = r.add_child(1);
        let p2 = p1.add_child(2);
        to_kv(&[p1, p2, p2.add_child(1)][..])
    }

    fn root_with_three_levels_two_siblings_on_level_2() -> Vec<(Key, Task)> {
        let r = Key::default();
        let p1 = r.add_child(1);
        let p11 = p1.add_child(1);
        let p12 = p1.add_child(2);
        to_kv(&[p1, p11, p11.add_child(1), p12, p12.add_child(1)][..])
    }

    #[test]
    fn root_level() {
        let entries = root_with_two_children();
        assert_eq!(
            Key::adjacency(&entries, 0),
            Adjacency(AboveAndBelow, NotFound, NotFound, NotFound, NotFound, NotFound)
        );
        assert_eq!(
            Key::adjacency(&entries, 1),
            Adjacency(Above, NotFound, NotFound, NotFound, NotFound, NotFound)
        );
    }

    #[test]
    fn level_2_two_siblings() {
        let entries = root_with_two_children_with_two_children();
        assert_eq!(
            Key::adjacency(&entries, 0),
            Adjacency(AboveAndBelow, NotFound, NotFound, NotFound, NotFound, NotFound)
        );
        {
            assert_eq!(
                Key::adjacency(&entries, 1),
                Adjacency(AboveAndBelow, AboveAndBelow, NotFound, NotFound, NotFound, NotFound)
            );
            assert_eq!(
                Key::adjacency(&entries, 2),
                Adjacency(AboveAndBelow, Above, NotFound, NotFound, NotFound, NotFound)
            );
        }
        assert_eq!(
            Key::adjacency(&entries, 3),
            Adjacency(Above, NotFound, NotFound, NotFound, NotFound, NotFound)
        );
        {
            assert_eq!(
                Key::adjacency(&entries, 4),
                Adjacency(NotFound, AboveAndBelow, NotFound, NotFound, NotFound, NotFound)
            );
            assert_eq!(
                Key::adjacency(&entries, 5),
                Adjacency(NotFound, Above, NotFound, NotFound, NotFound, NotFound)
            );
        }
    }

    #[test]
    fn level_3_single_sibling() {
        let entries = root_with_three_levels();
        assert_eq!(
            Key::adjacency(&entries, 0),
            Adjacency(Above, NotFound, NotFound, NotFound, NotFound, NotFound)
        );
        {
            assert_eq!(
                Key::adjacency(&entries, 1),
                Adjacency(NotFound, Above, NotFound, NotFound, NotFound, NotFound)
            );
            {
                assert_eq!(
                    Key::adjacency(&entries, 2),
                    Adjacency(NotFound, NotFound, Above, NotFound, NotFound, NotFound)
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
                Adjacency(Above, NotFound, NotFound, NotFound, NotFound, NotFound)
            );
            {
                assert_eq!(
                    Key::adjacency(&entries, 1),
                    Adjacency(NotFound, AboveAndBelow, NotFound, NotFound, NotFound, NotFound)
                );
                {
                    assert_eq!(
                        Key::adjacency(&entries, 2),
                        Adjacency(NotFound, AboveAndBelow, Above, NotFound, NotFound, NotFound)
                    );
                }

                assert_eq!(
                    Key::adjacency(&entries, 3),
                    Adjacency(NotFound, Above, NotFound, NotFound, NotFound, NotFound)
                );
                {
                    assert_eq!(
                        Key::adjacency(&entries, 4),
                        Adjacency(NotFound, NotFound, Above, NotFound, NotFound, NotFound)
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
            (Key::default().add_child(0).add_child(0).add_child(1), Task::default()),
        );
        entries.sort_by_key(|v| v.0);
        assert_eq!(
            Key::adjacency(&entries, 0),
            Adjacency(AboveAndBelow, NotFound, NotFound, NotFound, NotFound, NotFound),
        );
        assert_eq!(
            Key::adjacency(&entries, 1),
            Adjacency(AboveAndBelow, NotFound, NotFound, NotFound, NotFound, NotFound)
        );
        assert_eq!(
            Key::adjacency(&entries, 2),
            Adjacency(Above, NotFound, NotFound, NotFound, NotFound, NotFound)
        );
    }
}
