//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Duplex;
use crate::Executor;

// Messages sent through either endpoint arrive at the other one.
#[test]
fn test() {
  let (a, b) = Duplex::<u8>::pair();
  let mut executor = Executor::new();
  let received_by_a = executor.spawn(async move {
    a.tx.send(1).await.unwrap();
    a.rx.receive().await.unwrap()
  });
  let received_by_b = executor.spawn(async move {
    b.tx.send(2).await.unwrap();
    b.rx.receive().await.unwrap()
  });
  executor.run();
  assert_eq!(received_by_a.output(), Some(2));
  assert_eq!(received_by_b.output(), Some(1));
}
