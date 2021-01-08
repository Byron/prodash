// Copied and adapted from https://github.com/smol-rs/smol/blob/15447d6859df65fd1992f761ee46067bed62f8a5/src/spawn.rs
use std::future::Future;
use std::panic::catch_unwind;
use std::thread;

use async_executor::Executor;
pub use async_executor::Task;
use futures_lite::future;
use once_cell::sync::Lazy;

pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> Task<T> {
    static GLOBAL: Lazy<Executor<'_>> = Lazy::new(|| {
        thread::Builder::new()
            .name("smol-one".into())
            .spawn(|| loop {
                catch_unwind(|| async_io::block_on(GLOBAL.run(future::pending::<()>()))).ok();
            })
            .expect("cannot spawn executor thread");

        Executor::new()
    });

    GLOBAL.spawn(future)
}
