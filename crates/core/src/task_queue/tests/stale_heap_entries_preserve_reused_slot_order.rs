//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use super::super::TaskQueue;
use super::task;
use crate::Moment;

// A stale heap entry cannot give a reused slot its previous task's
// priority, even when both tasks were scheduled for the same moment.
#[test]
fn test() {
  let mut tasks = TaskQueue::default();
  let future = Moment::from_nanos(100);
  tasks.insert(future, 0, task);
  let old = tasks.key(0);
  tasks.insert(future, 1, task);
  tasks.reschedule(old, Moment::ZERO);
  let first = tasks.pop().map(|(_, task)| task.key);
  assert_eq!(first, Some(old));
  tasks.remove(old);

  tasks.insert(future, 2, task);
  assert_eq!(old.index(), tasks.key(2).index());
  let second = tasks.pop().map(|(moment, task)| (moment, task.key.id));
  assert_eq!(second, Some((future, 1)));
  let third = tasks.pop().map(|(moment, task)| (moment, task.key.id));
  assert_eq!(third, Some((future, 2)));
  let repeated = tasks.pop();
  assert!(repeated.is_none());
}
