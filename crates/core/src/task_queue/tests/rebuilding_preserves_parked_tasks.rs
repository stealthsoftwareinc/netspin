//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use super::super::QUEUE_REBUILD_SLACK;
use super::super::TaskQueue;
use super::task;
use crate::Moment;

// Rebuilding stale priority entries leaves parked tasks parked and
// available for a later wake.
#[test]
fn test() {
  let mut tasks = TaskQueue::default();
  tasks.insert(Moment::ZERO, 0, task);
  let parked = tasks.pop().map(|(_, task)| task.key.id);
  assert_eq!(parked, Some(0));
  let count = u64::try_from(QUEUE_REBUILD_SLACK * 2).unwrap();
  tasks.insert(Moment::from_nanos(count + 1), 1, task);
  for moment in (1..=count).rev() {
    tasks.reschedule(tasks.key(1), Moment::from_nanos(moment));
  }
  let scheduled =
    tasks.pop().map(|(moment, task)| (moment, task.key.id));
  assert_eq!(scheduled, Some((Moment::from_nanos(1), 1)));
  tasks.remove(tasks.key(1));
  let idle = tasks.pop();
  assert!(idle.is_none());
  tasks.wake(tasks.key(0), Moment::from_nanos(10));
  let woken = tasks.pop().map(|(moment, task)| (moment, task.key.id));
  assert_eq!(woken, Some((Moment::from_nanos(10), 0)));
}
