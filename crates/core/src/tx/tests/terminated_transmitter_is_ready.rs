//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::poll;

// A terminated channel's transmitter is ready to report its error.
#[test]
fn test() {
  let (tx, rx) = Channel::new().pair();
  drop(rx);
  let mut executor = Executor::new();
  let sender = executor.spawn(async move {
    assert_eq!(poll(&[&tx]).await, 0);
    tx.try_send(1).unwrap_err().cause
  });
  executor.run();

  assert_eq!(sender.output(), Some(ChannelError::Closed));
}
