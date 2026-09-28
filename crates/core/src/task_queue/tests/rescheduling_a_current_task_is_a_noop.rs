//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::TaskQueue;
use super::task;
use crate::moment::Moment;

// Rescheduling cannot delay a task queued for the current moment.
#[test]
fn test() {
  let current = Moment::ZERO + Duration::from_nanos(10);
  let later = current + Duration::from_nanos(10);
  let mut tasks = TaskQueue::default();
  tasks.insert(current, 0, task);

  let (_, current_task) = tasks.pop().unwrap();
  let key = current_task.key;
  tasks.schedule(key, current);
  tasks.reschedule(tasks.key(0), later);

  let popped = tasks.pop().map(|(moment, _)| moment);
  assert_eq!(popped, Some(current));
}
