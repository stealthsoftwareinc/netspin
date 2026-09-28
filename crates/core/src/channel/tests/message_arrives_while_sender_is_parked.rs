//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::ChannelOrder;
use crate::Executor;
use crate::Moment;
use crate::SendOptions;
use crate::Task;

// A message is delivered properly even when its sender is parked
// forever, and the run ends with the sender still parked.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::<i64>::new()
    .capacity(None)
    .local(false)
    .order(ChannelOrder::Unordered)
    .pair();
  let (dead_tx, dead_rx) =
    Channel::<i64>::new().capacity(None).local(false).pair();
  let sender = executor.spawn(async move {
    let _dead_tx = dead_tx;
    let options = SendOptions::new().latency(Duration::from_nanos(50));
    tx.send_with_options(1, options).await.unwrap();
    dead_rx.receive().await.unwrap();
  });
  let receiver = executor.spawn(async move {
    let message = rx.receive().await.unwrap();
    (message, Task::now())
  });
  executor.run();
  assert_eq!(sender.output(), None);
  assert_eq!(receiver.output(), Some((1, Moment::from_nanos(50))));
}
