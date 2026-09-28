//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use super::super::TaskQueue;
use super::task;
use crate::Moment;

// A parked task's first notification keeps the wake's position among
// current events before scheduling its future poll. Skipping the empty
// poll must not move it ahead of an intervening task at that future time.
#[test]
fn test() {
  let mut tasks = TaskQueue::default();
  tasks.insert(Moment::ZERO, 0, task);
  let parked = tasks.pop().map(|(_, task)| task.key);
  let parked = parked.unwrap();
  let now = Moment::from_nanos(5);
  let later = Moment::from_nanos(10);
  tasks.insert(now, 1, task);
  tasks.insert(now, 2, task);

  let sender = tasks.pop().map(|(_, task)| task.key);
  assert_eq!(sender, Some(tasks.key(1)));
  tasks.reschedule(parked, later);

  let intervening = tasks.pop().map(|(_, task)| task.key);
  let intervening = intervening.unwrap();
  assert_eq!(intervening.id, 2);
  tasks.schedule(intervening, later);

  let first = tasks.pop().map(|(moment, task)| (moment, task.key.id));
  assert_eq!(first, Some((later, 2)));
  let second = tasks.pop().map(|(moment, task)| (moment, task.key.id));
  assert_eq!(second, Some((later, 0)));
}
