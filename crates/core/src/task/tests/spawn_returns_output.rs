//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Executor;
use crate::Task;

// A task spawned by another task runs and makes its output available.
#[test]
fn test() {
  let mut executor = Executor::new();
  let parent = executor.spawn(async {
    let child = Task::spawn(async { 7 });
    (child, Task::spawn(async { "hi" }))
  });
  executor.run();
  let (a, b) = parent.output().unwrap();
  assert_eq!(a.output(), Some(7));
  assert_eq!(b.output(), Some("hi"));
}
