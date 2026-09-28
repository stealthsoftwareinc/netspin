//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Attach;
use crate::Channel;
use crate::Executor;
use crate::loss::DuplexLossBlock;

// DuplexLossBlock constructors and cloning configure no loss in both
// directions.
#[test]
fn test() {
  for block in [DuplexLossBlock::new(), DuplexLossBlock::default()] {
    let block = block.clone();
    let mut executor = Executor::new();
    let (tx_input, tx_target) = Channel::new().pair();
    let tx_output = block.tx.attach(&mut executor, tx_target);
    let (rx_input, rx_target) = Channel::new().pair();
    let rx_output = block.rx.attach(&mut executor, rx_target);
    let output = executor.spawn(async move {
      tx_input.send(1).await.unwrap();
      rx_input.send(2).await.unwrap();
      (
        tx_output.receive().await.unwrap(),
        rx_output.receive().await.unwrap(),
      )
    });
    executor.run();
    assert_eq!(output.output(), Some((1, 2)));
  }
}
