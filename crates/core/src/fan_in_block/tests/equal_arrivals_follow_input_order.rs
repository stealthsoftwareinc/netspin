//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Executor;
use crate::FanInBlock;

// Messages that are ready on several inputs at the same moment are
// forwarded in ascending input order.
#[test]
fn test() {
  let mut executor = Executor::new();
  let FanInBlock { app_txs, net_rx } =
    FanInBlock::<_>::new(&mut executor, 3);
  let mut app_txs = app_txs.into_iter();
  let app_tx0 = app_txs.next().unwrap();
  let app_tx1 = app_txs.next().unwrap();
  let app_tx2 = app_txs.next().unwrap();
  executor.spawn(async move {
    app_tx2.send(30).await.unwrap();
    app_tx0.send(10).await.unwrap();
    app_tx1.send(20).await.unwrap();
  });
  let receiver = executor.spawn(async move {
    [
      net_rx.receive().await.unwrap(),
      net_rx.receive().await.unwrap(),
      net_rx.receive().await.unwrap(),
    ]
  });
  executor.run();
  assert_eq!(receiver.output(), Some([10, 20, 30]));
}
