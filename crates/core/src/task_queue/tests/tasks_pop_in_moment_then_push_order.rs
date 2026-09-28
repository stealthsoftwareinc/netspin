//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::TaskQueue;
use super::task;
use crate::moment::Moment;

// Tasks pop in moment order with equal moments broken by push order.
#[test]
fn test() {
  let earlier = Moment::ZERO + Duration::from_nanos(10);
  let later = Moment::ZERO + Duration::from_nanos(20);
  let mut tasks = TaskQueue::default();
  tasks.insert(later, 0, task);
  tasks.insert(earlier, 1, task);
  tasks.insert(earlier, 2, task);
  let popped = tasks.pop().map(|(moment, task)| (moment, task.key.id));
  assert_eq!(popped, Some((earlier, 1)),);
  let popped = tasks.pop().map(|(moment, task)| (moment, task.key.id));
  assert_eq!(popped, Some((earlier, 2)),);
  let popped = tasks.pop().map(|(moment, task)| (moment, task.key.id));
  assert_eq!(popped, Some((later, 0)),);
}
