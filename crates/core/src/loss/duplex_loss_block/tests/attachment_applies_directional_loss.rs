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

// Attaching a DuplexLossBlock applies its directional loss to
// different transmitted and received message types.
#[test]
fn test() {
  let (target, peer) = Duplex::<u8, &'static str>::pair();
  let mut executor = Executor::new();
  let output = DuplexLossBlock::new()
    .tx(LossBlock::new().model(FixedLoss::new(1.0)))
    .rx(LossBlock::new().model(FixedLoss::new(0.0)))
    .attach(&mut executor, target);
  let options = ReceiveOptions::new().timeout(Duration::from_nanos(1));
  let received_by_output = executor.spawn(async move {
    output.tx.send(1).await.unwrap();
    let received = output.rx.receive_with_options(options).await;
    Task::sleep(Duration::from_nanos(1)).await;
    received
  });
  let received_by_peer = executor.spawn(async move {
    peer.tx.send("two").await.unwrap();
    let received = peer.rx.receive_with_options(options).await;
    Task::sleep(Duration::from_nanos(1)).await;
    received
  });
  executor.run();

  assert_eq!(received_by_output.output(), Some(Ok("two")));
  assert_eq!(
    received_by_peer.output(),
    Some(Err(ChannelError::TimedOut)),
  );
}
