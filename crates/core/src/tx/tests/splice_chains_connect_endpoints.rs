//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use std::rc::Rc;

use crate::Channel;
use crate::Executor;
use crate::Splice;

// Successive splices keep the remaining endpoints directly connected
// to one channel state.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (mut tx, rx) = Channel::new().pair();
  for _ in 0..32 {
    let (next_tx, next_rx) = Channel::new().pair();
    tx.splice(next_rx);
    tx = next_tx;
    assert!(Rc::ptr_eq(&tx.channel(), rx.channel_ref()));
  }
  executor.spawn(async move {
    tx.send(7).await.unwrap();
  });
  let output =
    executor.spawn(async move { rx.receive().await.unwrap() });
  executor.run();
  assert_eq!(output.output(), Some(7));
}
