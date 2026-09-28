//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::TaskQueue;
use super::task;
use crate::moment::Moment;

// Pushing a task before the current moment panics.
#[test]
#[should_panic(
  expected = "TaskQueue::schedule(): Task was scheduled before the current \
              moment."
)]
fn test() {
  let earlier = Moment::ZERO + Duration::from_nanos(10);
  let current = Moment::ZERO + Duration::from_nanos(20);
  let mut tasks = TaskQueue::default();
  tasks.insert(current, 0, task);
  let popped = tasks.pop().unwrap().0;
  assert_eq!(popped, current);
  tasks.insert(earlier, 1, task);
}
