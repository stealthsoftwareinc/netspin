//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::Cell;
use core::time::Duration;
use std::rc::Rc;

use crate::Attach;
use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::Task;
use crate::latency::LatencyBlock;
use crate::latency::LatencyModel;
use crate::latency::LatencyOverflow;
use crate::loss::LossModel;

#[derive(Clone)]
struct DropFirstAndFourth(Rc<Cell<usize>>);

impl LossModel for DropFirstAndFourth {
  fn should_drop(&mut self) -> bool {
    let index = self.0.get();
    self.0.set(index + 1);
    matches!(index, 0 | 3)
  }
}

#[derive(Clone)]
struct CountLatencies(Rc<Cell<usize>>);

impl LatencyModel for CountLatencies {
  fn next_latency(&mut self) -> Duration {
    self.0.set(self.0.get() + 1);
    Duration::from_nanos(50)
  }
}

// Loss is checked for every input, including when the buffer is full.
// Only surviving messages advance the latency model or occupy buffer
// space, under both tail-drop and backpressure overflow policies.
#[test]
fn test() {
  for overflow in
    [LatencyOverflow::TailDrop, LatencyOverflow::Backpressure]
  {
    let mut executor = Executor::new();
    let losses = Rc::new(Cell::new(0));
    let latencies = Rc::new(Cell::new(0));
    let (input, target) = Channel::new().pair();
    let output = LatencyBlock::new()
      .capacity(Some(1))
      .loss_model(DropFirstAndFourth(losses.clone()))
      .model(CountLatencies(latencies.clone()))
      .overflow(overflow)
      .attach(&mut executor, target);
    let sender = executor.spawn(async move {
      for message in 0..6 {
        input.send(message).await.unwrap();
      }
    });
    let receiver = executor.spawn(async move {
      let mut messages = Vec::new();
      loop {
        match output.receive().await {
          Ok(message) => {
            messages
              .push((message, Task::now().to_duration().as_nanos()));
          }
          Err(error) => {
            assert_eq!(error, ChannelError::Closed);
            break;
          }
        }
      }
      messages
    });
    executor.run();
    assert!(sender.output().is_some());
    assert_eq!(losses.get(), 6);
    assert_eq!(latencies.get(), 4);
    let expected = match overflow {
      LatencyOverflow::TailDrop => vec![(1, 50)],
      LatencyOverflow::Backpressure => {
        vec![(1, 50), (2, 100), (4, 150), (5, 200)]
      }
    };
    assert_eq!(receiver.output().unwrap(), expected);
  }
}
