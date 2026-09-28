//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use super::super::Signal;
use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::Pollable;
use crate::SendOptions;
use crate::Task;
use crate::poll;

// A signal reschedules a poll sleeping for a later channel arrival.
#[test]
fn test() {
  let (trigger, signal) = Signal::pair();
  let (tx, rx) = Channel::new().local(false).pair();
  let mut executor = Executor::new();
  executor.spawn(async move {
    tx.send_with_options(
      (),
      SendOptions::new().latency(Duration::from_nanos(100)),
    )
    .await
    .unwrap();
  });
  let output = executor.spawn(async move {
    let pollables: [&dyn Pollable; 2] = [&rx, &signal];
    (poll(&pollables).await, Task::now())
  });
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(50)).await;
    trigger.trigger();
  });
  executor.run();
  assert_eq!(output.output(), Some((1, Moment::from_nanos(50))));
}
