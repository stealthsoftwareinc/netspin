//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::ReceiveOptions;
use crate::SendOptions;
use crate::Task;

// Receive readiness wins when a message arrives at the timeout.
#[test]
fn test() {
  let (tx, rx) = Channel::new().local(false).pair();
  let mut executor = Executor::new();
  executor.spawn(async move {
    tx.send_with_options(
      7,
      SendOptions::new().latency(Duration::from_nanos(10)),
    )
    .await
    .unwrap();
  });
  let receiver = executor.spawn(async move {
    let message = rx
      .receive_with_options(
        ReceiveOptions::new().timeout(Duration::from_nanos(10)),
      )
      .await
      .unwrap();
    (message, Task::now())
  });
  executor.run();

  assert_eq!(receiver.output(), Some((7, Moment::from_nanos(10))),);
}
