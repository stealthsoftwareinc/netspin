//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::TaskQueue;
use super::task;
use crate::moment::Moment;

// A task rescheduled to the current moment retains its original push
// order relative to current-moment tasks.
#[test]
fn test() {
  let current = Moment::ZERO + Duration::from_nanos(10);
  let later = current + Duration::from_nanos(10);
  let mut tasks = TaskQueue::default();
  tasks.insert(later, 0, task);
  tasks.insert(current, 1, task);

  let (_, current_task) = tasks.pop().unwrap();
  let key = current_task.key;
  tasks.schedule(key, current);
  tasks.reschedule(tasks.key(0), current);

  let popped = tasks.pop().map(|(_, task)| task.key.id);
  assert_eq!(popped, Some(0));
  let popped = tasks.pop().map(|(_, task)| task.key.id);
  assert_eq!(popped, Some(1));
}
