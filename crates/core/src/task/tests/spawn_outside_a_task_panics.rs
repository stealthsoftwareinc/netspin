//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Task;

// Task::spawn() can only be called by a running task.
#[test]
#[should_panic(
  expected = "Task::spawn() must only be called within a task"
)]
fn test() {
  let _ = Task::spawn(async {});
}
