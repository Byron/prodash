use prodash::Progress;

#[test]
fn dyn_safe() {
    fn needs_mut_dyn(_p: &mut dyn Progress) {}
    fn needs_dyn(_p: &dyn Progress) {}
    let root = prodash::tree::Root::new();
    let mut child = root.add_child("hello");
    needs_mut_dyn(&mut child);
    needs_dyn(&child);
    let mut child_of_child = child.add_child("there");
    needs_mut_dyn(&mut child_of_child);
    needs_dyn(&child);
}

#[test]
fn thread_safe() {
    fn needs_send_sync<'a, T: Sync + Send + 'a>(_p: T) {}
    let root = prodash::tree::Root::new();
    let mut child = root.add_child("hello");
    needs_send_sync(&child);
    let child_of_child = child.add_child("there");
    needs_send_sync(&child_of_child);
    needs_send_sync(child_of_child);
    needs_send_sync(child);
}
