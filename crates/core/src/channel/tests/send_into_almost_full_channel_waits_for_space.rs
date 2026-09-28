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

// A send into an almost-full channel waits for enough space, even if
// the channel is never full.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::<usize>::new()
    .capacity(Some(10))
    .local(false)
    .message_size(|&m| m)
    .order(ChannelOrder::Unordered)
    .pair();
  executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_nanos(100));
    tx.send_with_options(4, options).await.unwrap();
    let options = SendOptions::new().latency(Duration::from_nanos(200));
    tx.send_with_options(5, options).await.unwrap();
    let options = SendOptions::new().latency(Duration::from_nanos(10));
    tx.send_with_options(6, options).await.unwrap();
  });
  let receiver = executor.spawn(async move {
    let mut log = Vec::new();
    for _ in 0..3 {
      let message = rx.receive().await.unwrap();
      log.push((message, Task::now()));
    }
    log
  });
  executor.run();
  assert_eq!(
    receiver.output().unwrap(),
    [
      (4, Moment::from_nanos(100)),
      (5, Moment::from_nanos(200)),
      (6, Moment::from_nanos(210))
    ]
  );
}
