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
use crate::loss::LossModel;

#[derive(Clone)]
struct DropFirst(bool);

impl LossModel for DropFirst {
  fn should_drop(&mut self) -> bool {
    core::mem::take(&mut self.0)
  }
}

// Clones maintain independent loss state and apply the configured loss
// and latency through receiver, transmitter, and pair attachment.
#[test]
fn test() {
  let mut executor = Executor::new();
  let block = LatencyBlock::new()
    .loss_model(DropFirst(true))
    .model(FixedLatency::new(Duration::from_nanos(50)));
  let mut receivers = Vec::new();
  for attachment in 0..3 {
    let block = block.clone();
    let (input, output) = match attachment {
      0 => {
        let (input, target) = Channel::new().pair();
        (input, block.attach(&mut executor, target))
      }
      1 => {
        let (target, output) = Channel::new().pair();
        (block.attach(&mut executor, target), output)
      }
      _ => {
        let (input, source) = Channel::new().pair();
        let (target, output) = Channel::new().pair();
        block.attach(&mut executor, (source, target));
        (input, output)
      }
    };
    receivers.push(executor.spawn(async move {
      input.send(0).await.unwrap();
      input.send(1).await.unwrap();
      drop(input);
      let message = output.receive().await.unwrap();
      let moment = Task::now();
      assert_eq!(output.receive().await, Err(ChannelError::Closed));
      (message, moment)
    }));
  }
  executor.run();
  for receiver in receivers {
    assert_eq!(receiver.output(), Some((1, Moment::from_nanos(50))));
  }
}
