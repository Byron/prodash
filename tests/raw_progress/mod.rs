use prodash::RawProgress;

#[test]
fn dyn_safe() {
    fn needs_dyn(_p: &mut dyn RawProgress) {}
    let root = prodash::tree::Root::new();
    let mut child = root.add_child("hello");
    needs_dyn(&mut child);
    let mut child_of_child = child.add_child("there");
    needs_dyn(&mut child_of_child);
}

#[test]
fn thread_safe() {
    fn needs_send<'a, T: Send + 'a>(_p: T) {}
    let root = prodash::tree::Root::new();
    let mut child = root.add_child("hello");
    needs_send(&child);
    let child_of_child = child.add_child("there");
    needs_send(&child_of_child);
}
