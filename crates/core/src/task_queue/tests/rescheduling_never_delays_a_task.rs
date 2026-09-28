//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::TaskQueue;
use super::task;
use crate::moment::Moment;

// Rescheduling never delays a task that is already scheduled earlier.
#[test]
fn test() {
  let scheduled = Moment::ZERO + Duration::from_nanos(10);
  let later = Moment::ZERO + Duration::from_nanos(20);
  let mut tasks = TaskQueue::default();
  tasks.insert(scheduled, 0, task);
  tasks.reschedule(tasks.key(0), later);
  let popped = tasks.pop().map(|(moment, _)| moment);
  assert_eq!(popped, Some(scheduled));
}
