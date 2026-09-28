//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Attach;
use crate::ChannelError;
use crate::Duplex;
use crate::Executor;
use crate::ReceiveOptions;
use crate::Task;
use crate::loss::DuplexLossBlock;
use crate::loss::FixedLoss;
use crate::loss::LossBlock;

// Attaching a DuplexLossBlock between duplex endpoints applies its
// directional loss to both message streams.
#[test]
fn test() {
  let (a, block_right) = Duplex::<u8, &'static str>::pair();
  let (block_left, b) = Duplex::<u8, &'static str>::pair();
  let mut executor = Executor::new();
  DuplexLossBlock::new()
    .tx(LossBlock::new().model(FixedLoss::new(1.0)))
    .rx(LossBlock::new().model(FixedLoss::new(0.0)))
    .attach(&mut executor, (block_left, block_right));
  let options = ReceiveOptions::new().timeout(Duration::from_nanos(1));
  let received_by_a = executor.spawn(async move {
    a.tx.send(1).await.unwrap();
    let received = a.rx.receive_with_options(options).await;
    Task::sleep(Duration::from_nanos(1)).await;
    received
  });
  let received_by_b = executor.spawn(async move {
    b.tx.send("two").await.unwrap();
    let received = b.rx.receive_with_options(options).await;
    Task::sleep(Duration::from_nanos(1)).await;
    received
  });
  executor.run();

  assert_eq!(received_by_a.output(), Some(Ok("two")));
  assert_eq!(received_by_b.output(), Some(Err(ChannelError::TimedOut)),);
}
