//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::TaskQueue;
use super::task;
use crate::moment::Moment;

// Rescheduling a task before the current moment panics.
#[test]
#[should_panic(
  expected = "TaskQueue::reschedule(): Task was scheduled before the \
              current moment."
)]
fn test() {
  let earlier = Moment::ZERO + Duration::from_nanos(10);
  let current = Moment::ZERO + Duration::from_nanos(20);
  let future = Moment::ZERO + Duration::from_nanos(30);
  let mut tasks = TaskQueue::default();
  tasks.insert(future, 0, task);
  tasks.insert(current, 1, task);
  let popped = tasks.pop().unwrap().0;
  assert_eq!(popped, current);
  tasks.reschedule(tasks.key(0), earlier);
}
