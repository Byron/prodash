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

    #[test]
    fn root_level_upwards() {
        assert_eq!(
            Key::adjecency(&root_with_two_children(), 1),
            Adjacency::default()
        )
    }
    #[test]
    fn root_level_downwards() {
        assert_eq!(
            Key::adjecency(&root_with_two_children(), 0),
            Adjacency::default()
        )
    }

    #[test]
    fn level_2_upwards() {
        assert_eq!(
            Key::adjecency(&root_with_two_children_with_two_children(), 1),
            Adjacency(Above, NotFound, NotFound, NotFound)
        );
        assert_eq!(
            Key::adjecency(&root_with_two_children_with_two_children(), 2),
            Adjacency(Above, NotFound, NotFound, NotFound)
        );
    }
}
