//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;
use crate::Splice;

// A splice chain retains the upstream-most channel's capacity.
#[test]
fn test() {
  let (upstream_tx, mut upstream_rx) =
    Channel::new().capacity(Some(3)).pair();
  for capacity in [2, 1] {
    let (downstream_tx, downstream_rx) =
      Channel::new().capacity(Some(capacity)).pair();
    upstream_rx.splice(downstream_tx);
    upstream_rx = downstream_rx;
  }
  let mut executor = Executor::new();
  let free_space = executor.spawn(async move {
    let before = upstream_tx.free_space();
    for message in [1, 2, 3] {
      upstream_tx.expect_send(message).unwrap();
    }
    (before, upstream_tx.free_space())
  });
  let received = executor.spawn(async move {
    let mut messages = Vec::new();
    for _ in 0..3 {
      messages.push(upstream_rx.receive().await.unwrap());
    }
    messages
  });
  executor.run();
  assert_eq!(free_space.output(), Some((3, 0)));
  assert_eq!(received.output(), Some(vec![1, 2, 3]));
}
