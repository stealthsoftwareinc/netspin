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

// In an unordered channel, a low-latency message sent after a
// high-latency message overtakes it.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::<i64>::new()
    .capacity(None)
    .local(false)
    .order(ChannelOrder::Unordered)
    .pair();
  executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_nanos(100));
    tx.send_with_options(1, options).await.unwrap();
    Task::sleep(Duration::from_nanos(20)).await;
    let options = SendOptions::new().latency(Duration::from_nanos(10));
    tx.send_with_options(2, options).await.unwrap();
  });
  let received = executor.spawn(async move {
    let mut log = Vec::new();
    for _ in 0..2 {
      let message = rx.receive().await.unwrap();
      log.push((message, Task::now()));
    }
    log
  });
  executor.run();
  assert_eq!(
    received.output().unwrap(),
    [(2, Moment::from_nanos(30)), (1, Moment::from_nanos(100))]
  );
}
