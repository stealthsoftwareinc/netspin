//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::SendError;
use crate::test_error::TestError;

// Dropping the receiver interrupts a backpressured send.
#[test]
fn test() {
  let (tx, rx) = Channel::<u8, TestError>::default().pair();
  let mut executor = Executor::new();
  let sender = executor.spawn(async move {
    tx.send(1).await.unwrap();
    tx.send(2).await
  });
  executor.spawn(async move {
    drop(rx);
  });
  executor.run();
  assert_eq!(
    sender.output(),
    Some(Err(SendError {
      cause: ChannelError::Closed,
      sent: 0,
      unsent: 2,
    })),
  );
}
