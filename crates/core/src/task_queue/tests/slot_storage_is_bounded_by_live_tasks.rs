//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::Cell;
use std::rc::Rc;

use super::super::TaskQueue;
use super::task;
use crate::Moment;

// Completed tasks release their futures immediately, and storage is
// reused even as task IDs increase and stale priority entries remain.
#[test]
fn test() {
  let mut tasks = TaskQueue::default();
  let value = Rc::new(Cell::new(0));
  for id in 0..4096 {
    let captured = value.clone();
    tasks.insert(Moment::from_nanos(10_000), id, |key| {
      let mut task = task(key);
      task.future = Box::pin(async move {
        captured.set(id);
      });
      task
    });
    let key = tasks.key(id);
    tasks.reschedule(key, Moment::ZERO);
    let popped = tasks.pop().map(|(_, task)| task.key);
    assert_eq!(popped, Some(key));
    tasks.remove(key);
    tasks.wake(key, Moment::ZERO);
    assert_eq!(tasks.slots.len(), 1);
    assert!(tasks.slots[0].is_none());
    assert_eq!(tasks.free_slots, [0]);
    assert_eq!(tasks.heap_tasks, 0);
    assert_eq!(Rc::strong_count(&value), 1);
  }
  let repeated = tasks.pop();
  assert!(repeated.is_none());
}
