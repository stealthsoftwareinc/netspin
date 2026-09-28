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
use crate::latency::LatencyOverflow;

// Backpressure waits for latency buffer space before admitting the
// next message.
#[test]
fn test() {
  let latency = Duration::from_millis(50);
  let mut executor = Executor::new();
  let (input, target) = Channel::new().pair();
  let output = LatencyBlock::new()
    .capacity(Some(1))
    .model(FixedLatency::new(latency))
    .overflow(LatencyOverflow::Backpressure)
    .attach(&mut executor, target);
  let received = executor.spawn(async move {
    input.send(1).await.unwrap();
    input.send(2).await.unwrap();
    let mut received = Vec::new();
    for _ in 0..2 {
      let message = output.receive().await.unwrap();
      received.push((message, Task::now().to_duration()));
    }
    received
  });
  executor.run();
  assert_eq!(
    received.output().unwrap(),
    [(1, latency), (2, latency + latency)],
  );
}
