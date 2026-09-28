//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::QUEUE_REBUILD_SLACK;
use super::super::TaskQueue;
use super::task;
use crate::moment::Moment;

// Rebuilding the priority queue leaves current-moment tasks intact.
#[test]
fn test() {
  let current = Moment::ZERO + Duration::from_nanos(10);
  let reschedules = QUEUE_REBUILD_SLACK * 2;
  let future = current
    + Duration::from_nanos(u64::try_from(reschedules + 1).unwrap());
  let mut tasks = TaskQueue::default();
  tasks.insert(future, 0, task);
  tasks.insert(current, 1, task);

  let (_, current_task) = tasks.pop().unwrap();
  let key = current_task.key;
  tasks.schedule(key, current);
  for nanoseconds in (1..=reschedules).rev() {
    tasks.reschedule(
      tasks.key(0),
      current
        + Duration::from_nanos(u64::try_from(nanoseconds).unwrap()),
    );
  }

  let popped = tasks.pop().map(|(_, task)| task.key.id);
  assert_eq!(popped, Some(1));
  let popped = tasks.pop().map(|(_, task)| task.key.id);
  assert_eq!(popped, Some(0));
  let popped = tasks.pop();
  assert!(popped.is_none());
}
