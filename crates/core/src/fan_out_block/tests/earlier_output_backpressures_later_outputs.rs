//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Executor;
use crate::FanOutBlock;
use crate::Moment;
use crate::Task;

// An earlier fan-out output backpressures all later outputs.
#[test]
fn test() {
  let mut executor = Executor::new();
  let FanOutBlock {
    app_tx,
    mut net_rxs,
  } = FanOutBlock::<_>::new(&mut executor, 2);
  let first = net_rxs.remove(0);
  let second = net_rxs.remove(0);
  executor.spawn(async move {
    Task::sleep(Duration::from_nanos(100)).await;
    [
      first.receive().await.unwrap(),
      first.receive().await.unwrap(),
    ]
  });
  let later = executor.spawn(async move {
    (
      [
        second.receive().await.unwrap(),
        second.receive().await.unwrap(),
      ],
      Task::now(),
    )
  });
  executor.spawn(async move {
    app_tx.send(1).await.unwrap();
    app_tx.send(2).await.unwrap();
  });
  executor.run();
  assert_eq!(
    later.output(),
    Some(([1, 2], Moment::ZERO + Duration::from_nanos(100),)),
  );
}
