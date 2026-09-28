//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use std::thread;

use tokio::runtime::Handle;

use crate::Executor;
use crate::Task;

fn runtime_id() -> tokio::runtime::Id {
  let mut executor = Executor::new();
  let output = executor
    .spawn(async { Task::block_on(async { Handle::current().id() }) });
  executor.run();
  output.output().unwrap()
}

// Executors running on different threads share the block-on runtime.
#[test]
fn test() {
  let a = thread::spawn(runtime_id);
  let b = thread::spawn(runtime_id);

  assert_eq!(a.join().unwrap(), b.join().unwrap());
}
