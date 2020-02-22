use criterion::*;

use prodash::{tree::MessageLevel, Tree, TreeOptions};

fn usage(c: &mut Criterion) {
    fn small_tree() -> Tree {
        TreeOptions {
            initial_capacity: 10,
            message_buffer_capacity: 2,
        }
        .create()
    };
    c.benchmark_group("Tree::add_child")
        .throughput(Throughput::Elements(4))
        .bench_function(
            "add children to build a tree of tasks and clear them (in drop)",
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
    c.benchmark_group("tree::Item::set")
        .throughput(Throughput::Elements(5))
        .bench_function("set tree 5 times", |b| {
            let root = small_tree();
            let mut progress = root.add_child("the one");
            progress.init(Some(20), Some("element"));
            b.iter(|| {
                progress.set(1);
                progress.set(2);
                progress.set(3);
                progress.set(4);
                progress.set(5);
            });
        });
    c.benchmark_group("tree::Item::message")
        .throughput(Throughput::Elements(1))
        .bench_function(
            "send one message with a full message buffer (worst case performance)",
            |b| {
                let root = small_tree();
                let mut progress = root.add_child("the one");
                progress.init(Some(20), Some("element"));
                b.iter(|| {
                    progress.message(MessageLevel::Success, "for testing");
                });
            },
        );
    c.benchmark_group("Tree::copy_messages")
        .throughput(Throughput::Elements(2))
        .bench_function("copy all messages with buffer being at capacity", |b| {
            let root = small_tree();
            let mut progress = root.add_child("the one");
            progress.init(Some(20), Some("element"));
            progress.done("foo");
            progress.done("bar");
            let mut out = Vec::new();
            b.iter(|| {
                root.copy_messages(&mut out);
            });
        });
}

criterion_group!(benches, usage);
criterion_main!(benches);
