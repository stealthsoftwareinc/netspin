//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Executor;

// A spawned task's output is retrievable after the task is complete,
// and tasks can have heterogeneous output types.
#[test]
fn test() {
  let mut executor = Executor::new();
  let a = executor.spawn(async { 7 });
  let b = executor.spawn(async { "hi" });
  let c = executor.spawn(async {});
  executor.run();
  assert_eq!(a.output(), Some(7));
  assert_eq!(b.output(), Some("hi"));
  assert_eq!(c.output(), Some(()));
}
