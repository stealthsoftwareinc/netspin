//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::TaskQueue;
use super::task;
use crate::moment::Moment;

// Tasks repushed for the current moment keep cycling in push order.
#[test]
fn test() {
  let moment = Moment::ZERO + Duration::from_nanos(10);
  let mut tasks = TaskQueue::default();
  tasks.insert(moment, 0, task);
  tasks.insert(moment, 1, task);

  let (popped_moment, first) = tasks.pop().unwrap();
  assert_eq!((popped_moment, first.key.id), (moment, 0));
  let key = first.key;
  tasks.schedule(key, moment);

  let (popped_moment, second) = tasks.pop().unwrap();
  assert_eq!((popped_moment, second.key.id), (moment, 1));
  let key = second.key;
  tasks.schedule(key, moment);

  let popped = tasks.pop().map(|(_, task)| task.key.id);
  assert_eq!(popped, Some(0));
  let popped = tasks.pop().map(|(_, task)| task.key.id);
  assert_eq!(popped, Some(1));
}
