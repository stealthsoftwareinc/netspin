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
use crate::poll;

// A parked poll can have its wakeup brought forward repeatedly by
// either another channel or an overtaking message on the first channel.
#[test]
fn test() {
  let mut executor = Executor::new();
  let channel = Channel::new()
    .capacity(None)
    .local(false)
    .order(ChannelOrder::Unordered);
  let (first_tx, first_rx) = channel.clone().pair();
  let (second_tx, second_rx) = channel.pair();
  let receiver = executor.spawn(async move {
    let mut received = Vec::new();
    for _ in 0..3 {
      let key = poll(&[&first_rx, &second_rx]).await;
      let rx = if key == 0 { &first_rx } else { &second_rx };
      let value = rx.expect_receive().unwrap();
      received.push((key, value, Task::now()));
    }
    received
  });
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(5)).await;
    first_tx
      .send_with_options(
        1,
        SendOptions::new().latency(Duration::from_nanos(95)),
      )
      .await
      .unwrap();
    Task::sleep(Duration::from_nanos(10)).await;
    first_tx
      .send_with_options(
        3,
        SendOptions::new().latency(Duration::from_nanos(5)),
      )
      .await
      .unwrap();
    Task::sleep_until(Moment::from_nanos(110)).await;
  });
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    second_tx
      .send_with_options(
        2,
        SendOptions::new().latency(Duration::from_nanos(50)),
      )
      .await
      .unwrap();
    Task::sleep_until(Moment::from_nanos(110)).await;
  });
  executor.run();

  let received = receiver.output().unwrap();
  assert_eq!(
    received,
    vec![
      (0, 3, Moment::from_nanos(20)),
      (1, 2, Moment::from_nanos(60)),
      (0, 1, Moment::from_nanos(100)),
    ],
  );
}
