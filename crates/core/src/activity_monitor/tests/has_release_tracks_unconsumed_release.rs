//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Activity;
use crate::Executor;
use crate::Task;
use crate::poll;

// ActivityMonitor reports a release only until it is consumed.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (activity, monitor) = Activity::pair();
  executor.spawn(async move {
    let lease = activity.lease();
    Task::work(Duration::from_nanos(5)).await;
    drop(lease);
  });
  let observer = executor.spawn(async move {
    assert!(!monitor.has_release());
    assert_eq!(poll(&[&monitor]).await, 0);
    let ready = monitor.has_release();
    let idle = monitor.expect_release();
    (ready, idle, monitor.has_release())
  });
  executor.run();

  assert_eq!(observer.output(), Some((true, true, false)));
}
