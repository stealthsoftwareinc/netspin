//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Duplex;
use crate::Executor;

// A duplex pair cross-connects different transmitted and received
// message types.
#[test]
fn test() {
  let (a, b) = Duplex::<u8, &'static str>::pair();
  let mut executor = Executor::new();
  let received_by_a = executor.spawn(async move {
    a.tx.send(1).await.unwrap();
    a.rx.receive().await.unwrap()
  });
  let received_by_b = executor.spawn(async move {
    b.tx.send("two").await.unwrap();
    b.rx.receive().await.unwrap()
  });
  executor.run();

  assert_eq!(received_by_a.output(), Some("two"));
  assert_eq!(received_by_b.output(), Some(1));
}
