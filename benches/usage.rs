use glassbench::*;

use prodash::{messages::MessageLevel, Tree, TreeOptions};

fn usage(b: &mut Bench) {
    fn small_tree() -> Tree {
        TreeOptions {
            initial_capacity: 10,
            message_buffer_capacity: 2,
        }
        .create()
    }
    b.task(
        "Tree::add_child: add children to build a tree of tasks and clear them (in drop)",
        |b| {
            let root = small_tree();
            b.iter(|| {
                let mut c = root.add_child("1");
                let _one = c.add_child("1");
                let _two = c.add_child("2");
                let _three = c.add_child("3");
            });
        },
    );
    b.task("tree::Item::set: set tree 5 times", |b| {
        let root = small_tree();
        let mut progress = root.add_child("the one");
        progress.init(Some(20), Some("element".into()));
        b.iter(|| {
            progress.set(1);
            progress.set(2);
            progress.set(3);
            progress.set(4);
            progress.set(5);
        });
    });
    b.task(
        "tree::Item::message: send one message with a full message buffer (worst case performance)",
        |b| {
            let root = small_tree();
            let mut progress = root.add_child("the one");
            progress.init(Some(20), Some("element".into()));
            b.iter(|| {
                progress.message(MessageLevel::Success, "for testing");
            });
        },
    );
    b.task(
        "Tree::copy_messages: copy all messages with buffer being at capacity",
        |b| {
            let root = small_tree();
            let mut progress = root.add_child("the one");
            progress.init(Some(20), Some("element".into()));
            progress.done("foo");
            progress.done("bar");
            let mut out = Vec::new();
            b.iter(|| {
                root.copy_messages(&mut out);
            });
        },
    );
}

glassbench!("standard usage", usage,);
