//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;
use crate::Splice;

// Splicing a nonempty upstream channel panics.
#[test]
#[should_panic(
  expected = "Splicing a channel requires it to be empty."
)]
fn test() {
  let mut executor = Executor::new();
  let (upstream_tx, upstream_rx) = Channel::new().pair();
  let (downstream_tx, downstream_rx) = Channel::new().pair();
  executor.spawn(async move {
    let _downstream_rx = downstream_rx;
    upstream_tx.send(7).await.unwrap();
    downstream_tx.splice(upstream_rx);
  });
  executor.run();
}
