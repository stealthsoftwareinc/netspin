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

// An observer polling the end of a forwarding pipeline sees each
// message become available at exactly the right moment, including a
// message whose delivery to the forwarder was rescheduled.
#[test]
fn test() {
  let mut executor = Executor::new();
  let unordered = Channel::<i64>::new()
    .capacity(None)
    .local(false)
    .order(ChannelOrder::Unordered);
  let (tx1, rx1) = unordered.clone().pair();
  let (tx2, rx2) = unordered.pair();
  executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_nanos(100));
    tx1.send_with_options(1, options).await.unwrap();
    Task::sleep(Duration::from_nanos(20)).await;
    let options = SendOptions::new().latency(Duration::from_nanos(10));
    tx1.send_with_options(2, options).await.unwrap();
  });
  executor.spawn(async move {
    for _ in 0..2 {
      let message = rx1.receive().await.unwrap();
      tx2.send(message).await.unwrap();
    }
  });
  let observer = executor.spawn(async move {
    let mut log = Vec::new();
    for moment in [29, 31, 99, 101] {
      Task::sleep_until(Moment::from_nanos(moment)).await;
      log.push((Task::now(), rx2.try_receive().unwrap()));
    }
    log
  });
  executor.run();
  assert_eq!(
    observer.output().unwrap(),
    [
      (Moment::from_nanos(29), None),
      (Moment::from_nanos(31), Some(2)),
      (Moment::from_nanos(99), None),
      (Moment::from_nanos(101), Some(1))
    ]
  );
}
