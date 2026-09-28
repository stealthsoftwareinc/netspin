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
use crate::loss::LossBlock;
use crate::loss::LossModel;

#[derive(Clone)]
struct DropFirst {
  first: bool,
}

impl LossModel for DropFirst {
  fn should_drop(&mut self) -> bool {
    core::mem::replace(&mut self.first, false)
  }
}

// Attaching a LossBlock to a receiver applies its configured loss
// model to each message.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (input, target) = Channel::new().pair();
  let output = LossBlock::new()
    .model(DropFirst { first: true })
    .attach(&mut executor, target);
  executor.spawn(async move {
    input.send(0).await.unwrap();
    input.send(1).await.unwrap();
    Task::sleep(Duration::from_nanos(2)).await;
  });
  let received = executor.spawn(async move {
    let message = output.receive().await.unwrap();
    let options =
      ReceiveOptions::new().timeout(Duration::from_nanos(1));
    assert_eq!(
      output.receive_with_options(options).await,
      Err(ChannelError::TimedOut),
    );
    message
  });
  executor.run();
  assert_eq!(received.output(), Some(1));
}
