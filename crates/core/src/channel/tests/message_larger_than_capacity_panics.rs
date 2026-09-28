//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;

// Sending a message that's larger than the channel capacity panics if
// the channel has no message_split function, as this means the message
// can never fit.
#[test]
#[should_panic(expected = "must not be impossibly large")]
fn test() {
  let mut executor = Executor::new();
  let (tx, _rx) = Channel::<usize>::new()
    .capacity(Some(10))
    .local(false)
    .message_size(|&m| m)
    .pair();
  executor.spawn(async move {
    tx.send(11).await.unwrap();
  });
  executor.run();
}
