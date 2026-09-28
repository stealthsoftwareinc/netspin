//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Attach;
use crate::Channel;
use crate::Executor;
use crate::Task;
use crate::latency::DuplexLatencyBlock;
use crate::latency::FixedLatency;
use crate::latency::LatencyBlock;

// Transmit and receive latency blocks can be configured independently.
#[test]
fn test() {
  let tx_latency = Duration::from_millis(10);
  let rx_latency = Duration::from_millis(20);
  let block = DuplexLatencyBlock::<u8, &'static str>::new()
    .tx(LatencyBlock::new().model(FixedLatency::new(tx_latency)))
    .rx(LatencyBlock::new().model(FixedLatency::new(rx_latency)));
  let mut executor = Executor::new();
  let (tx_input, tx_target) = Channel::new().pair();
  let tx_output = block.tx.attach(&mut executor, tx_target);
  let (rx_input, rx_target) = Channel::new().pair();
  let rx_output = block.rx.attach(&mut executor, rx_target);
  let output = executor.spawn(async move {
    tx_input.send(1).await.unwrap();
    rx_input.send("two").await.unwrap();
    let tx = (tx_output.receive().await.unwrap(), Task::now());
    let rx = (rx_output.receive().await.unwrap(), Task::now());
    (tx, rx)
  });
  executor.run();

  assert_eq!(
    output.output().map(|((tx, tx_moment), (rx, rx_moment))| (
      (tx, tx_moment.to_duration()),
      (rx, rx_moment.to_duration()),
    )),
    Some(((1, tx_latency), ("two", rx_latency))),
  );
}
