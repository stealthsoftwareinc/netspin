//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Activity;
use crate::Executor;
use crate::Task;
use crate::poll;

// Dropping a lease makes its monitor ready at the release moment.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (activity, monitor) = Activity::pair();
  let consumer = executor.spawn(async move {
    let lease = activity.lease();
    Task::work(Duration::from_nanos(5)).await;
    drop(lease);
  });
  let observer = executor.spawn(async move {
    assert_eq!(poll(&[&monitor]).await, 0);
    (Task::now(), monitor.expect_release(), monitor.is_active())
  });
  executor.run();

  assert!(consumer.output().is_some());
  assert_eq!(
    observer.output(),
    Some((crate::Moment::from_nanos(5), true, false))
  );
}
