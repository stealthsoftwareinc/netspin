//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use std::rc::Rc;

use crate::Channel;
use crate::Executor;
use crate::poll;

// Polling and sending through a transmitter from different tasks
// panics.
#[test]
#[should_panic(expected = "A channel must not have multiple senders")]
fn test() {
  let (tx, _rx) = Channel::new().pair();
  let tx = Rc::new(tx);
  let mut executor = Executor::new();
  executor.spawn({
    let tx = tx.clone();
    async move {
      assert_eq!(poll(&[tx.as_ref()]).await, 0);
    }
  });
  executor.spawn(async move {
    tx.expect_send(1).unwrap();
  });
  executor.run();
}
