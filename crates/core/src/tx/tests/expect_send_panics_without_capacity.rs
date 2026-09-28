//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;

// Tx::expect_send() panics when the channel lacks capacity.
#[test]
#[should_panic(
  expected = "The channel must have enough free space to send the message"
)]
fn test() {
  let mut executor = Executor::new();
  let (tx, _rx) = Channel::new().pair();
  executor.spawn(async move {
    tx.expect_send(1).unwrap();
    tx.expect_send(2).unwrap();
  });
  executor.run();
}
