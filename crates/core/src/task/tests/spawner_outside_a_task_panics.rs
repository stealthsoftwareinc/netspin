//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Task;

// Task::spawner() can only be called by a running task.
#[test]
#[should_panic(
  expected = "Task::spawner() must only be called within a task"
)]
fn test() {
  let _ = Task::spawner();
}
