//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;
use crate::Splice;

// Splicing channels with unequal capacities retains the upstream
// channel's capacity.
#[test]
fn test() {
  let (upstream_tx, upstream_rx) =
    Channel::new().capacity(Some(2)).pair();
  let (downstream_tx, downstream_rx) = Channel::new().pair();
  downstream_tx.splice(upstream_rx);
  let mut executor = Executor::new();
  let free_space = executor.spawn(async move {
    let before = upstream_tx.free_space();
    upstream_tx.expect_send(7).unwrap();
    (before, upstream_tx.free_space())
  });
  let received = executor
    .spawn(async move { downstream_rx.receive().await.unwrap() });
  executor.run();
  assert_eq!(free_space.output(), Some((2, 1)));
  assert_eq!(received.output(), Some(7));
}
