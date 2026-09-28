//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use super::super::TaskQueue;
use super::task;
use crate::Moment;

// Waking a parked task schedules it once; later wakes neither repeat
// it nor interrupt an explicit sleep.
#[test]
fn test() {
  let mut tasks = TaskQueue::default();
  tasks.insert(Moment::ZERO, 0, task);
  let initial = tasks.pop().map(|(moment, task)| (moment, task.key.id));
  assert_eq!(initial, Some((Moment::ZERO, 0)));
  let parked = tasks.pop();
  assert!(parked.is_none());

  tasks.wake(tasks.key(0), Moment::from_nanos(10));
  tasks.wake(tasks.key(0), Moment::from_nanos(20));
  let woken = tasks.pop().map(|(moment, task)| {
    task.moment = moment;
    (moment, task.key.id)
  });
  assert_eq!(woken, Some((Moment::from_nanos(10), 0)));

  tasks.schedule(tasks.key(0), Moment::from_nanos(30));
  tasks.wake(tasks.key(0), Moment::from_nanos(15));
  let slept = tasks.pop().map(|(moment, task)| (moment, task.key.id));
  assert_eq!(slept, Some((Moment::from_nanos(30), 0)));
  let repeated = tasks.pop();
  assert!(repeated.is_none());
}
