//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Executor;
use crate::FanOutBlock;

// Every fan-out output receives every input message.
#[test]
fn test() {
  let mut executor = Executor::new();
  let FanOutBlock { app_tx, net_rxs } =
    FanOutBlock::<_>::new(&mut executor, 3);
  let received = executor.spawn(async move {
    app_tx.send(123).await.unwrap();
    let mut messages = Vec::new();
    for net_rx in net_rxs {
      messages.push(net_rx.receive().await.unwrap());
    }
    messages
  });
  executor.run();
  assert_eq!(received.output(), Some(vec![123, 123, 123]));
}
