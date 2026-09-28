//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;

// Rx::try_receive_up_to() rejects an oversized ready message when the
// channel has no message splitter.
#[test]
#[should_panic(
  expected = "netspin_core::Rx::try_receive_up_to(): The ready message \
              exceeds the maximum size, but the channel has no message \
              splitter."
)]
fn test() {
  let (tx, rx) = Channel::new()
    .capacity(Some(8))
    .message_size(Vec::len)
    .pair();
  let mut executor = Executor::new();
  executor.spawn(async move {
    tx.send(vec![1, 2, 3, 4]).await.unwrap();
    let _ = rx.try_receive_up_to(2);
  });
  executor.run();
}
