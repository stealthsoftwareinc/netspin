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

// Attaching a DuplexLatencyBlock applies its directional latencies to
// different transmitted and received message types.
#[test]
fn test() {
  let tx_latency = Duration::from_millis(10);
  let rx_latency = Duration::from_millis(20);
  let (target, peer) = Duplex::<u8, &'static str>::pair();
  let mut executor = Executor::new();
  let output = DuplexLatencyBlock::new()
    .tx(LatencyBlock::new().model(FixedLatency::new(tx_latency)))
    .rx(LatencyBlock::new().model(FixedLatency::new(rx_latency)))
    .attach(&mut executor, target);
  let received_by_output = executor.spawn(async move {
    output.tx.send(1).await.unwrap();
    let message = output.rx.receive().await.unwrap();
    (message, Task::now())
  });
  let received_by_peer = executor.spawn(async move {
    peer.tx.send("two").await.unwrap();
    let message = peer.rx.receive().await.unwrap();
    (message, Task::now())
  });
  executor.run();

  assert_eq!(
    received_by_output
      .output()
      .map(|(message, moment)| (message, moment.to_duration())),
    Some(("two", rx_latency)),
  );
  assert_eq!(
    received_by_peer
      .output()
      .map(|(message, moment)| (message, moment.to_duration())),
    Some((1, tx_latency)),
  );
}
