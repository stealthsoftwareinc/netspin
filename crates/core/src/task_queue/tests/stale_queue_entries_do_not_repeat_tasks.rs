//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::TaskQueue;
use super::task;
use crate::moment::Moment;

// A stale heap entry left by rescheduling cannot pop its task twice.
#[test]
fn test() {
  let earlier = Moment::ZERO + Duration::from_nanos(10);
  let later = Moment::ZERO + Duration::from_nanos(20);
  let mut tasks = TaskQueue::default();
  tasks.insert(later, 0, task);
  tasks.reschedule(tasks.key(0), earlier);
  let popped = tasks.pop().map(|(_, task)| task.key.id);
  assert_eq!(popped, Some(0));
  let popped = tasks.pop();
  assert!(popped.is_none());
}
