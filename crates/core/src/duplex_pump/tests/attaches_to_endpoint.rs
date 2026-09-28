//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Attach;
use crate::Duplex;
use crate::DuplexPump;
use crate::Executor;

// DuplexPump attaches to an endpoint with different transmitted and
// received message types.
#[test]
fn test() {
  let (target, peer) = Duplex::<u8, &'static str>::pair();
  let mut executor = Executor::new();
  let output = DuplexPump::new().attach(&mut executor, target);
  let received_by_output = executor.spawn(async move {
    output.tx.send(1).await.unwrap();
    output.rx.receive().await.unwrap()
  });
  let received_by_peer = executor.spawn(async move {
    peer.tx.send("two").await.unwrap();
    peer.rx.receive().await.unwrap()
  });
  executor.run();

  assert_eq!(received_by_output.output(), Some("two"));
  assert_eq!(received_by_peer.output(), Some(1));
}
