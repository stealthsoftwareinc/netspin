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

// both() configures the transmit and receive latency blocks alike.
#[test]
fn test() {
  let latency = Duration::from_millis(10);
  let block = DuplexLatencyBlock::<u8>::new()
    .both(LatencyBlock::new().model(FixedLatency::new(latency)));
  let mut executor = Executor::new();
  let (tx_input, tx_target) = Channel::new().pair();
  let tx_output = block.tx.attach(&mut executor, tx_target);
  let (rx_input, rx_target) = Channel::new().pair();
  let rx_output = block.rx.attach(&mut executor, rx_target);
  let output = executor.spawn(async move {
    tx_input.send(1).await.unwrap();
    rx_input.send(2).await.unwrap();
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
    Some(((1, latency), (2, latency))),
  );
}
