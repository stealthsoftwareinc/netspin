//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use super::super::QUEUE_REBUILD_SLACK;
use super::super::TaskQueue;
use super::task;
use crate::Moment;

// A heap rebuild preserves an initial wake still waiting in the FIFO,
// its order relative to a rescheduled event, and another future event
// that was not rescheduled during the rebuild.
#[test]
fn test() {
  let mut tasks = TaskQueue::default();
  tasks.insert(Moment::ZERO, 0, task);
  let parked = tasks.pop().map(|(_, task)| task.key).unwrap();
  let later = Moment::from_nanos(20);
  tasks.reschedule(parked, later);

  let count = u64::try_from(QUEUE_REBUILD_SLACK * 2).unwrap();
  tasks.insert(Moment::from_nanos(count + 20), 1, task);
  tasks.insert(Moment::from_nanos(10), 2, task);
  for moment in (20..count + 20).rev() {
    tasks.reschedule(tasks.key(1), Moment::from_nanos(moment));
  }
  assert!(tasks.queue.len() <= QUEUE_REBUILD_SLACK + 4);
  let first = tasks.pop().map(|(moment, task)| (moment, task.key.id));
  let second = tasks.pop().map(|(moment, task)| (moment, task.key.id));
  let third = tasks.pop().map(|(moment, task)| (moment, task.key.id));
  assert_eq!(first, Some((Moment::from_nanos(10), 2)));
  assert_eq!(second, Some((later, 1)));
  assert_eq!(third, Some((later, 0)));
  let remaining = tasks.pop();
  assert!(remaining.is_none());
}
