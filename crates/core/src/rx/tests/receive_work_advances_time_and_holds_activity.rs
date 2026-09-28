//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Activity;
use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::Task;
use crate::poll;

// Modeled receive work advances the consumer while its activity is
// held and notifies the monitor on completion.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (activity, monitor) = Activity::pair();
  let (tx, rx) = Channel::new().pair();
  let rx = rx.work_on_receive(activity, |_| Duration::from_nanos(5));
  executor.spawn(async move {
    tx.send(7).await.unwrap();
  });
  let receiver = executor.spawn(async move {
    let message = rx.receive().await.unwrap();
    (message, Task::now())
  });
  let observer = executor.spawn(async move {
    assert!(monitor.is_active());
    let active_at = Task::now();
    assert_eq!(poll(&[&monitor]).await, 0);
    (active_at, monitor.expect_release(), Task::now())
  });
  executor.run();

  assert_eq!(receiver.output(), Some((7, Moment::from_nanos(5))));
  assert_eq!(
    observer.output(),
    Some((Moment::ZERO, true, Moment::from_nanos(5)))
  );
}
