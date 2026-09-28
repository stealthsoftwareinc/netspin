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

// A send into a totally full channel waits for enough space if space
// becomes available before the timeout.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::<i64>::new()
    .capacity(Some(1))
    .local(false)
    .order(ChannelOrder::Unordered)
    .pair();
  let sender = executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_nanos(100));
    tx.send_with_options(1, options).await.unwrap();
    let options = SendOptions::new()
      .latency(Duration::from_nanos(10))
      .timeout(Duration::from_nanos(200));
    tx.send_with_options(2, options).await.unwrap();
    Task::now()
  });
  let receiver = executor.spawn(async move {
    let mut log = Vec::new();
    for _ in 0..2 {
      let message = rx.receive().await.unwrap();
      log.push((message, Task::now()));
    }
    log
  });
  executor.run();
  assert_eq!(sender.output().unwrap(), Moment::from_nanos(100));
  assert_eq!(
    receiver.output().unwrap(),
    [(1, Moment::from_nanos(100)), (2, Moment::from_nanos(110))]
  );
}
