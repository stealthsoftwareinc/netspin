//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::QUEUE_REBUILD_RATIO;
use super::super::QUEUE_REBUILD_SLACK;
use super::super::TaskQueue;
use super::task;
use crate::moment::Moment;

// Repeated rescheduling rebuilds the heap before stale entries can grow
// without bound.
#[test]
fn test() {
  let mut tasks = TaskQueue::default();
  let reschedules = QUEUE_REBUILD_SLACK * 2;
  let initial = Moment::ZERO
    + Duration::from_nanos(u64::try_from(reschedules + 1).unwrap());
  tasks.insert(initial, 0, task);
  for nanoseconds in (1..=reschedules).rev() {
    tasks.reschedule(
      tasks.key(0),
      Moment::ZERO
        + Duration::from_nanos(u64::try_from(nanoseconds).unwrap()),
    );
  }
  assert!(
    tasks.queue_len() <= QUEUE_REBUILD_RATIO + QUEUE_REBUILD_SLACK,
  );
}
