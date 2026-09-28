//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use super::super::TaskQueue;
use super::task;
use crate::Moment;

// Notifications received before a parked task's wake is processed
// retain the earliest arrival and cause only one poll.
#[test]
fn test() {
  let mut tasks = TaskQueue::default();
  tasks.insert(Moment::ZERO, 0, task);
  let parked = tasks.pop().map(|(_, task)| task.key).unwrap();
  tasks.reschedule(parked, Moment::from_nanos(100));
  tasks.reschedule(parked, Moment::from_nanos(20));
  tasks.reschedule(parked, Moment::from_nanos(60));

  let received = tasks.pop().map(|(moment, task)| (moment, task.key));
  assert_eq!(received, Some((Moment::from_nanos(20), parked)));
  let repeated = tasks.pop();
  assert!(repeated.is_none());
}
