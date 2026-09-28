//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Attach;
use crate::Channel;
use crate::Executor;
use crate::Task;
use crate::latency::FixedLatency;
use crate::latency::LatencyBlock;

// Attaching a LatencyBlock between a receiver and transmitter applies
// its fixed latency to each message.
#[test]
fn test() {
  let latency = Duration::from_millis(50);
  let mut executor = Executor::new();
  let (input, block_input) = Channel::new().pair();
  let (block_output, output) = Channel::new().pair();
  LatencyBlock::new()
    .model(FixedLatency::new(latency))
    .attach(&mut executor, (block_input, block_output));
  let received = executor.spawn(async move {
    input.send(1).await.unwrap();
    let message = output.receive().await.unwrap();
    (message, Task::now())
  });
  executor.run();
  assert_eq!(
    received
      .output()
      .map(|(message, moment)| (message, moment.to_duration())),
    Some((1, latency)),
  );
}
