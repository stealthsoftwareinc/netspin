//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Activity;
use crate::Executor;
use crate::Task;
use crate::poll;

// A release observed after a newer lease starts does not report the
// activity idle.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (activity, monitor) = Activity::pair();
  executor.spawn(async move {
    drop(activity.lease());
    let lease = activity.lease();
    Task::work(Duration::from_nanos(5)).await;
    drop(lease);
  });
  let observer = executor.spawn(async move {
    assert_eq!(poll(&[&monitor]).await, 0);
    let first = monitor.expect_release();
    assert_eq!(poll(&[&monitor]).await, 0);
    let second = monitor.expect_release();
    (first, second)
  });
  executor.run();

  assert_eq!(observer.output(), Some((false, true)));
}
