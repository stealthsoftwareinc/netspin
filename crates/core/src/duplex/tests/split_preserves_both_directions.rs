//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Duplex;
use crate::Executor;
use crate::Split as _;

// Splitting a duplex endpoint preserves both directions.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (endpoint, peer) = Duplex::<u8, &str>::pair();
  let (tx, rx) = endpoint.split();
  let endpoint = executor.spawn(async move {
    tx.send(1).await.unwrap();
    rx.receive().await.unwrap()
  });
  let peer = executor.spawn(async move {
    peer.tx.send("hello").await.unwrap();
    peer.rx.receive().await.unwrap()
  });
  executor.run();
  assert_eq!(endpoint.output(), Some("hello"));
  assert_eq!(peer.output(), Some(1));
}
