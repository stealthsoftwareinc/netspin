//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;
use crate::poll;

// A transmitter is ready with any free space even when a particular
// message cannot fit.
#[test]
fn test() {
  let (tx, _rx) = Channel::new()
    .capacity(Some(3))
    .message_size(Vec::len)
    .pair();
  let mut executor = Executor::new();
  let sender = executor.spawn(async move {
    tx.expect_send(vec![1, 2]).unwrap();
    assert_eq!(poll(&[&tx]).await, 0);
    tx.try_send(vec![3, 4]).unwrap()
  });
  executor.run();

  assert_eq!(sender.output(), Some(Some(vec![3, 4])));
}
