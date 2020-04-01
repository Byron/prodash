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
        assert_eq!(
            Key::adjacency(&root_with_two_children(), 0),
            Adjacency::default()
        );
        assert_eq!(
            Key::adjacency(&root_with_two_children(), 1),
            Adjacency::default()
        );
    }

    #[test]
    fn level_2_two_siblings() {
        let entries = root_with_two_children_with_two_children();
        assert_eq!(
            Key::adjacency(&entries, 0),
            Adjacency(NotFound, NotFound, NotFound, NotFound)
        );
        {
            assert_eq!(
                Key::adjacency(&entries, 1),
                Adjacency(AboveAndBelow, NotFound, NotFound, NotFound)
            );
            assert_eq!(
                Key::adjacency(&entries, 2),
                Adjacency(Above, NotFound, NotFound, NotFound)
            );
        }
        assert_eq!(
            Key::adjacency(&entries, 3),
            Adjacency(NotFound, NotFound, NotFound, NotFound)
        );
        {
            assert_eq!(
                Key::adjacency(&entries, 4),
                Adjacency(AboveAndBelow, NotFound, NotFound, NotFound)
            );
            assert_eq!(
                Key::adjacency(&entries, 5),
                Adjacency(Above, NotFound, NotFound, NotFound)
            );
        }
    }

    #[test]
    fn level_3_single_sibling() {
        let entries = root_with_three_levels();
        assert_eq!(
            Key::adjacency(&entries, 0),
            Adjacency(NotFound, NotFound, NotFound, NotFound)
        );
        {
            assert_eq!(
                Key::adjacency(&entries, 1),
                Adjacency(Above, NotFound, NotFound, NotFound)
            );
            {
                assert_eq!(
                    Key::adjacency(&entries, 2),
                    Adjacency(NotFound, Above, NotFound, NotFound)
                );
            }
        }
    }

    #[test]
    fn level_3_two_siblings() {
        let entries = root_with_three_levels_two_siblings_on_level_2();
        {
            // assert_eq!(
            //     Key::adjacency(&entries, 0),
            //     Adjacency(NotFound, NotFound, NotFound, NotFound)
            // );
            {
                // assert_eq!(
                //     Key::adjacency(&entries, 1),
                //     Adjacency(AboveAndBelow, NotFound, NotFound, NotFound)
                // );
                {
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
                        Adjacency(NotFound, Above, NotFound, NotFound)
                    );
                }
            }
        }
    }
}
