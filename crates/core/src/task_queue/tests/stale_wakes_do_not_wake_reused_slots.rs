//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use super::super::TaskQueue;
use super::task;
use crate::Moment;

// A completed task's wake cannot affect a different task that reuses
// its storage slot, and stale heap entries do not repeat either task.
#[test]
fn test() {
  let mut tasks = TaskQueue::default();
  tasks.insert(Moment::from_nanos(100), 0, task);
  let old = tasks.key(0);
  tasks.reschedule(old, Moment::ZERO);
  let first = tasks.pop().map(|(_, task)| task.key);
  assert_eq!(first, Some(old));
  tasks.remove(old);

  tasks.insert(Moment::ZERO, 1, task);
  let new = tasks.key(1);
  assert_eq!(old.index(), new.index());
  assert_ne!(old.id, new.id);
  let second = tasks.pop().map(|(_, task)| task.key);
  assert_eq!(second, Some(new));

  tasks.wake(old, Moment::from_nanos(10));
  let unexpected = tasks.pop();
  assert!(unexpected.is_none());
  tasks.reschedule(new, Moment::from_nanos(20));
  let woken = tasks.pop().map(|(moment, task)| (moment, task.key));
  assert_eq!(woken, Some((Moment::from_nanos(20), new)));
  tasks.remove(new);
  let repeated = tasks.pop();
  assert!(repeated.is_none());
}
