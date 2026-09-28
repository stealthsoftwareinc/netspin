//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Attach;
use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::ReceiveOptions;
use crate::Task;
use crate::latency::FixedLatency;
use crate::latency::LatencyBlock;
use crate::latency::LatencyOverflow;

// Tail-drop overflow discards a message when the latency buffer is
// full.
#[test]
fn test() {
  let latency = Duration::from_millis(50);
  let mut executor = Executor::new();
  let (input, target) = Channel::new().pair();
  let output = LatencyBlock::new()
    .capacity(Some(1))
    .model(FixedLatency::new(latency))
    .overflow(LatencyOverflow::TailDrop)
    .attach(&mut executor, target);
  let received = executor.spawn(async move {
    input.send(1).await.unwrap();
    input.send(2).await.unwrap();
    let first = output.receive().await.unwrap();
    let options = ReceiveOptions::new().timeout(latency);
    let second = output.receive_with_options(options).await;
    (first, second, Task::now().to_duration())
  });
  executor.run();
  assert_eq!(
    received.output(),
    Some((1, Err(ChannelError::TimedOut), latency + latency)),
  );
}
