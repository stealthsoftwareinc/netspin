//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Task;

// Task::spawn_detached() can only be called by a running task.
#[test]
#[should_panic(
  expected = "Task::spawn_detached() must only be called within a task"
)]
fn test() {
  Task::spawn_detached(async {});
}
