//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;

// Tx::expect_send() sends when the channel has sufficient capacity.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::new().pair();
  executor.spawn(async move {
    tx.expect_send(7).unwrap();
  });
  let receiver =
    executor.spawn(async move { rx.receive().await.unwrap() });
  executor.run();
  assert_eq!(receiver.output(), Some(7));
}
