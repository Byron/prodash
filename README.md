**prodash** is a dashboard for displaying progress of concurrent applications.

It's easy to integrate thanks to a pragmatic API, and comes with a terminal user interface by default.

[![asciicast](https://asciinema.org/a/301838.svg)](https://asciinema.org/a/301838)

## How to use…

Be sure to read the documentation at https://docs.rs/prodash, it contains various examples on how to get started.

Or run the demo application like so `cd prodash && cargo run --example dashboard`.

## Features

* fast insertions and updates for transparent progress tracking of highly concurrent programs
* a messages buffer for information about success and failure
* a terminal user interface for visualization, with keyboard controls and dynamic re-sizing
* unicode and multi-width character support

## Limitations

* it does copy quite some state each time it displays progress information and messages
* The underlying sync data structure, `dashmap`, does not document every use of unsafe
  * I also evaluated `evmap`, which has 25% less uses of unsafe, but a more complex interface.
  * Thus far it seemed 'ok' to use, who knows… we are getting mutable pieces of a hashmap from multiple threads,
    however, we never hand out multiple handles to the same child which should make actual concurrent access to 
    the same key impossible.
* If there are more than 2^16 tasks
  * then
    * running concurrently on a single level of the tree, they start overwriting each other
    * over its lifetime, even though they do not run concurrently, eventually new tasks will seem like old tasks (their ID wrapped around)
  * why
    * on drop, they never decrement a child count used to generate a new ID
  * fix
    * make the id bigger, like u32
    * we should do that once there is a performance test
    
## Lessons Learned

* `drop()` is not garantueed to be called when the future returns Ready and is in the futures::executor::ThreadPool
  * Workaround: drop and cleanup explicitly, prone to forgetting it.
  * This is also why `futures::future::abortable()` works (by stopping the polling), but doesn't as cleanup is not performed,
    even though it clearly would be preferred.
  * fix
    * Use a join handle and await it - this will drop the future properly
* `select()` might not work with complex futures - these should then be `boxed()` if `Unpin` isn't implemented.
