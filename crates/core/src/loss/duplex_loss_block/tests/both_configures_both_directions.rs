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
use crate::loss::DuplexLossBlock;
use crate::loss::FixedLoss;
use crate::loss::LossBlock;

// both() configures the transmit and receive loss blocks alike.
#[test]
fn test() {
  let block = DuplexLossBlock::new()
    .both(LossBlock::new().model(FixedLoss::new(1.0)));
  let mut executor = Executor::new();
  let (tx_input, tx_target) = Channel::new().pair();
  let tx_output = block.tx.attach(&mut executor, tx_target);
  let (rx_input, rx_target) = Channel::new().pair();
  let rx_output = block.rx.attach(&mut executor, rx_target);
  executor.spawn(async move {
    tx_input.send(1).await.unwrap();
    rx_input.send(2).await.unwrap();
    Task::sleep(Duration::from_nanos(2)).await;
  });
  let output = executor.spawn(async move {
    let options =
      ReceiveOptions::new().timeout(Duration::from_nanos(1));
    let tx = tx_output.receive_with_options(options).await;
    let rx = rx_output.receive_with_options(options).await;
    (tx, rx)
  });
  executor.run();
  assert_eq!(
    output.output(),
    Some((Err(ChannelError::TimedOut), Err(ChannelError::TimedOut),)),
  );
}
