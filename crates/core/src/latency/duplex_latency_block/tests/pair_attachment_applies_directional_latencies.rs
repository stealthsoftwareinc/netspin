//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Attach;
use crate::Duplex;
use crate::Executor;
use crate::Task;
use crate::latency::DuplexLatencyBlock;
use crate::latency::FixedLatency;
use crate::latency::LatencyBlock;

// Attaching a DuplexLatencyBlock between duplex endpoints applies its
// directional latencies to both message streams.
#[test]
fn test() {
  let tx_latency = Duration::from_millis(10);
  let rx_latency = Duration::from_millis(20);
  let (a, block_right) = Duplex::<u8, &'static str>::pair();
  let (block_left, b) = Duplex::<u8, &'static str>::pair();
  let mut executor = Executor::new();
  DuplexLatencyBlock::new()
    .tx(LatencyBlock::new().model(FixedLatency::new(tx_latency)))
    .rx(LatencyBlock::new().model(FixedLatency::new(rx_latency)))
    .attach(&mut executor, (block_left, block_right));
  let received_by_a = executor.spawn(async move {
    a.tx.send(1).await.unwrap();
    let message = a.rx.receive().await.unwrap();
    (message, Task::now())
  });
  let received_by_b = executor.spawn(async move {
    b.tx.send("two").await.unwrap();
    let message = b.rx.receive().await.unwrap();
    (message, Task::now())
  });
  executor.run();

  assert_eq!(
    received_by_a
      .output()
      .map(|(message, moment)| (message, moment.to_duration())),
    Some(("two", rx_latency)),
  );
  assert_eq!(
    received_by_b
      .output()
      .map(|(message, moment)| (message, moment.to_duration())),
    Some((1, tx_latency)),
  );
}
