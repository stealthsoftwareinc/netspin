//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Activity;
use crate::Executor;
use crate::Moment;
use crate::Signal;
use crate::Task;
use crate::poll;
use crate::poll_with_timeout;

// Repeated references to a signal or activity share the same polling
// task, with and without a timeout, and return the first index.
#[test]
fn test() {
  for timed in [false, true] {
    let mut executor = Executor::new();
    let (trigger, signal) = Signal::pair();
    let (activity, monitor) = Activity::pair();
    let receiver = executor.spawn(async move {
      let signal_key = if timed {
        poll_with_timeout(&[&signal, &signal], Duration::from_nanos(20))
          .await
          .unwrap()
      } else {
        poll(&[&signal, &signal]).await
      };
      let signal_at = Task::now();
      let activity_key = if timed {
        poll_with_timeout(
          &[&monitor, &monitor],
          Duration::from_nanos(20),
        )
        .await
        .unwrap()
      } else {
        poll(&[&monitor, &monitor]).await
      };
      let idle = monitor.expect_release();
      (signal_key, signal_at, activity_key, Task::now(), idle)
    });
    executor.spawn(async move {
      Task::sleep(Duration::from_nanos(5)).await;
      trigger.trigger();
      let lease = activity.lease();
      Task::work(Duration::from_nanos(5)).await;
      drop(lease);
    });
    executor.run();

    let received = receiver.output();
    assert_eq!(
      received,
      Some((0, Moment::from_nanos(5), 0, Moment::from_nanos(10), true)),
    );
  }
}
