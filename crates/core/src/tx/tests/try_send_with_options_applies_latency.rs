//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::SendOptions;
use crate::Task;

// Tx::try_send_with_options() applies the configured latency.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::new().local(false).pair();
  executor.spawn(async move {
    assert_eq!(
      tx.try_send_with_options(
        7,
        SendOptions::new().latency(Duration::from_nanos(100)),
      )
      .unwrap(),
      None,
    );
  });
  let receiver = executor.spawn(async move {
    let message = rx.receive().await.unwrap();
    (message, Task::now())
  });
  executor.run();
  assert_eq!(receiver.output(), Some((7, Moment::from_nanos(100))));
}
