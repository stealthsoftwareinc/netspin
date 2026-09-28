//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Executor;
use crate::FanInBlock;
use crate::Task;

// Messages arriving on different inputs are forwarded as one stream.
#[test]
fn test() {
  let mut executor = Executor::new();
  let FanInBlock { app_txs, net_rx } =
    FanInBlock::<_>::new(&mut executor, 2);
  let mut app_txs = app_txs.into_iter();
  let app_tx0 = app_txs.next().unwrap();
  let app_tx1 = app_txs.next().unwrap();
  executor.spawn(async move {
    app_tx1.send(17).await.unwrap();
    Task::sleep(Duration::from_nanos(100)).await;
    app_tx0.send(29).await.unwrap();
  });
  let receiver = executor.spawn(async move {
    [
      net_rx.receive().await.unwrap(),
      net_rx.receive().await.unwrap(),
    ]
  });
  executor.run();
  assert_eq!(receiver.output(), Some([17, 29]));
}
