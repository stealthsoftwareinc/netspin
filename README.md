# NetSpin

NetSpin is a high-fidelity sampling-based network modeling framework.
It is written in Rust as a custom async runtime.
It has three fundamental primitives:

- *Tasks* are coroutines that model individual processes.

- Tasks use *channels* to send messages to each other.

- The *executor* schedules and runs a set of tasks.

The executor virtualizes time across tasks and channels.
This is the key difference from normal async runtimes like Tokio.

Because NetSpin is built on top of async Rust, you can model a system as
a set of tasks that communicate through channels instead of manually
decomposing the system into state machines driven by I/O events.
Furthermore, the executor is single-threaded, tracks a separate virtual
time for each task, and always guarantees that the currently running
task is furthest behind in virtual time.
This greatly reduces temporal synchronization difficulties between tasks
and makes it easy to run multiple trials in parallel by using one
executor per thread.

## Crates

- [`netspin-core`](https://crates.io/crates/netspin-core)

## Examples

The following is the simplest example,
[`basic.rs`](https://github.com/stealthsoftwareinc/netspin/blob/main/crates/core/examples/basic.rs):

```rust
use core::time::Duration;

use netspin_core::Channel;
use netspin_core::Executor;
use netspin_core::Task;

fn main() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::new().pair();

  executor.spawn(async move {
    for message in ["Hello", "from", "Alice"] {
      Task::sleep(Duration::from_secs(1)).await;
      tx.send(message).await.unwrap();
    }
  });

  executor.spawn(async move {
    while let Ok(message) = rx.receive().await {
      println!("t={}: {message}", Task::now());
    }
  });

  executor.run();
}
```

Output:

```text
t=1: Hello
t=2: from
t=3: Alice
```

## Status

NetSpin is still experimental and has not reached version 1.0.0 yet, so
it may change significantly between releases.

NetSpin is developed in a private upstream repository, with public
releases made when the version number is bumped.
Issues are welcome, but pull requests are not accepted at this time.

## License

NetSpin is licensed under the Apache License, Version 2.0.
