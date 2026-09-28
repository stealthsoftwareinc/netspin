//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Executor;
use crate::Task;

#[test]
fn test() {
  assert!(!Task::is_running());

  let mut executor = Executor::new();
  let running = executor.spawn(async { Task::is_running() });
  executor.run();

  assert_eq!(running.output(), Some(true));
  assert!(!Task::is_running());
}
