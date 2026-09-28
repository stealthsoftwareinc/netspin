//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Attach;
use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::Moment;
use crate::Task;
use crate::latency::FixedLatency;
use crate::latency::LatencyBlock;
use crate::loss::FixedLoss;

// Dropping every message leaves no delayed work and propagates channel
// closure immediately.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (input, target) = Channel::<()>::new().pair();
  let output = LatencyBlock::new()
    .loss_model(FixedLoss::new(1.0))
    .model(FixedLatency::new(Duration::from_secs(1)))
    .attach(&mut executor, target);
  let sender = executor.spawn(async move {
    for _ in 0..3 {
      input.send(()).await.unwrap();
    }
  });
  let receiver = executor
    .spawn(async move { (output.receive().await, Task::now()) });
  executor.run();
  assert!(sender.output().is_some());
  assert_eq!(
    receiver.output(),
    Some((Err(ChannelError::Closed), Moment::ZERO)),
  );
}
