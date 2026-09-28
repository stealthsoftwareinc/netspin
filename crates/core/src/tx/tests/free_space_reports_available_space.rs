//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;

// Tx::free_space() reports the channel's currently available space.
#[test]
fn test() {
  let (tx, _rx) = Channel::new().capacity(Some(2)).pair();
  let mut executor = Executor::new();
  let free_space = executor.spawn(async move {
    assert_eq!(tx.free_space(), 2);
    tx.expect_send(7).unwrap();
    tx.free_space()
  });
  executor.run();
  assert_eq!(free_space.output(), Some(1));
}
