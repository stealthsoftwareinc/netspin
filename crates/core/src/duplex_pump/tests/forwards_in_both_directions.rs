//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Attach;
use crate::Duplex;
use crate::DuplexPump;
use crate::Executor;

// Messages are pumped between the duplex endpoints in both directions.
#[test]
fn test() {
  let (a, pump_a) = Duplex::<u8>::pair();
  let (b, pump_b) = Duplex::<u8>::pair();
  let mut executor = Executor::new();
  DuplexPump::new().attach(&mut executor, (pump_a, pump_b));
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
