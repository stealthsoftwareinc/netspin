//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Task;

// Task::block_on() can only be called by a running task.
#[test]
#[should_panic(
  expected = "Task::block_on() must only be called within a task"
)]
fn test() {
  Task::block_on(async {});
}
