//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::TaskQueue;
use super::task;
use crate::moment::Moment;

// Rescheduling preserves a task's original position relative to tasks
// scheduled for the same moment.
#[test]
fn test() {
  let earlier = Moment::ZERO + Duration::from_nanos(10);
  let later = Moment::ZERO + Duration::from_nanos(20);
  let mut tasks = TaskQueue::default();
  tasks.insert(later, 0, task);
  tasks.insert(earlier, 1, task);
  tasks.reschedule(tasks.key(0), earlier);
  let popped = tasks.pop().map(|(_, task)| task.key.id);
  assert_eq!(popped, Some(0));
  let popped = tasks.pop().map(|(_, task)| task.key.id);
  assert_eq!(popped, Some(1));
}
